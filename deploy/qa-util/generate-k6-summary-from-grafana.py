#!/usr/bin/env python3
import argparse
import json
from datetime import datetime, timezone
from collections import defaultdict

import numpy as np
import requests


def fetch_raw_counter_metric(grafana_url, api_token, metric_name, start_time, end_time):
    """
    Fetches raw counter values from Grafana Prometheus datasource.

    Args:
        grafana_url (str): The base URL of the Grafana instance.
        api_token (str): The API token for authentication.
        metric_name (str): The full metric name (e.g., metric_sum or metric_count).
        start_time (datetime): The start of the time range.
        end_time (datetime): The end of the time range.

    Returns:
        dict: A dictionary mapping (method, labels_key) to list of (timestamp, value) tuples.
    """
    headers = {
        "Authorization": f"Bearer {api_token}",
        "Content-Type": "application/json",
        "X-Grafana-Org-Id": "86",
    }

    start_ts_ms = int(start_time.timestamp() * 1000)
    end_ts_ms = int(end_time.timestamp() * 1000)

    labels = '{exported_service="wallet-provider", exported_namespace="logius-wallet-ont"}'
    query = f'{metric_name}{labels}'

    data = {
        "queries": [
            {
                "refId": "A",
                "expr": query,
                "range": True,
                "instant": False,
                "datasource": {
                    "type": "prometheus",
                    "uid": "logius-wallet-proxy"
                },
                "editorMode": "code",
                "legendFormat": "__auto",
                "intervalMs": 15000,
                "maxDataPoints": 1000
            }
        ],
        "from": str(start_ts_ms),
        "to": str(end_ts_ms),
    }

    api_url = f"{grafana_url}/api/ds/query"

    try:
        response = requests.post(api_url, headers=headers, json=data)
        response.raise_for_status()
        json_data = response.json()

        frames = json_data.get("results", {}).get("A", {}).get("frames", [])

        if not frames:
            print("  WARNING: No frames returned in response")
            print(f"  Response keys: {json_data.keys()}")
            if "results" in json_data:
                print(f"  Results keys: {json_data['results'].keys()}")

        series_data = {}

        for frame in frames:
            schema = frame.get("schema", {})
            fields = schema.get("fields", [])
            data_values = frame.get("data", {}).get("values", [])

            if len(fields) < 2 or len(data_values) < 2:
                continue

            # Get method and other labels
            value_field = fields[1]
            labels = value_field.get("labels", {})
            method = labels.get("method", "UNKNOWN")

            # Create a unique key for this series using only stable labels
            # Use method + status + path to uniquely identify each series
            # (other labels like pod name might differ between queries)
            status = labels.get("status", "")
            path = labels.get("path", "")
            ingress = labels.get("ingress", "")
            host = labels.get("host", "")

            labels_key = (method, status, path, ingress, host)

            # Get timestamps and values
            timestamps = data_values[0]
            values = data_values[1]

            # Store as list of (timestamp, value) tuples
            time_value_pairs = []
            for ts, val in zip(timestamps, values):
                if val is not None and not np.isnan(val):
                    time_value_pairs.append((ts, val))

            if time_value_pairs:
                # If this key already exists (multiple pods), we need to aggregate the data
                key = (method, labels_key)
                if key in series_data:
                    # Merge time series data from multiple pods by summing values at matching timestamps
                    existing_pairs = dict(series_data[key])
                    for ts, val in time_value_pairs:
                        if ts in existing_pairs:
                            existing_pairs[ts] += val
                        else:
                            existing_pairs[ts] = val
                    series_data[key] = sorted(existing_pairs.items())
                else:
                    series_data[key] = time_value_pairs

        print(f"  Parsed {len(series_data)} time series with data")
        for (method, _), pairs in list(series_data.items())[:3]:
            print(f"    - {method}: {len(pairs)} data points, first={pairs[0][1]:.3f}, last={pairs[-1][1]:.3f}")

        return series_data

    except requests.exceptions.RequestException as e:
        print(f"Error fetching metrics from Grafana: {e}")
        if 'response' in locals() and response:
            print(f"Response status: {response.status_code}")
            print(f"Response body: {response.text}")
        return {}
    except json.JSONDecodeError as e:
        print(f"Error decoding JSON from Grafana response: {e}")
        print(f"Response text: {response.text}")
        return {}


def is_failed_status(status_code):
    """
    Determine if a status code represents a failed request.

    Args:
        status_code (str): HTTP status code

    Returns:
        bool: True if status code is 4xx or 5xx, False otherwise
    """
    try:
        code = int(status_code)
        return code >= 400
    except (ValueError, TypeError):
        return False


def calculate_durations_from_histograms(sum_data, count_data):
    """
    Calculate average durations from _sum and _count histogram data.

    For each matching time series, calculate:
    - The increase in sum over the time range
    - The increase in count over the time range
    - Average duration = increase(sum) / increase(count)

    Args:
        sum_data (dict): Dictionary of (method, labels_key) -> [(timestamp, sum_value)]
        count_data (dict): Dictionary of (method, labels_key) -> [(timestamp, count_value)]

    Returns:
        tuple: (method_durations, request_stats)
            - method_durations: Dictionary of method -> [average_durations_in_ms]
            - request_stats: Dictionary with 'total', 'failed', 'success' counts
    """
    method_durations = defaultdict(list)
    total_requests = 0
    failed_requests = 0
    success_requests = 0

    print(f"  Matching {len(sum_data)} sum series with {len(count_data)} count series")

    matched = 0
    skipped_no_match = 0
    skipped_too_few_points = 0
    skipped_no_increase = 0

    # Match up sum and count series by their keys
    for key in sum_data.keys():
        if key not in count_data:
            skipped_no_match += 1
            continue

        matched += 1
        method, labels_key = key

        sum_series = sum_data[key]
        count_series = count_data[key]

        if len(sum_series) < 2 or len(count_series) < 2:
            skipped_too_few_points += 1
            print(f"    Skipping {method}: only {len(sum_series)} sum points, {len(count_series)} count points")
            continue

        sum_start = sum_series[0][1]
        sum_end = sum_series[-1][1]
        count_start = count_series[0][1]
        count_end = count_series[-1][1]

        sum_increase = sum_end - sum_start
        count_increase = count_end - count_start

        if count_increase > 0:
            avg_duration_seconds = sum_increase / count_increase
            avg_duration_ms = avg_duration_seconds * 1000

            # Add this average duration as a data point
            # We repeat the average for each request to weight the median calculation.
            # This means median is calculated across endpoint averages, not individual requests.
            # This is a limitation of using histogram data - we cannot recover the true distribution.
            num_requests = max(1, int(count_increase))
            for _ in range(num_requests):
                method_durations[method].append(avg_duration_ms)

            status = labels_key[1] if len(labels_key) > 1 else "unknown"

            total_requests += num_requests
            if is_failed_status(status):
                failed_requests += num_requests
            else:
                success_requests += num_requests

            print(f"    {method} {status}: {num_requests} requests, avg={avg_duration_ms:.2f}ms (sum_inc={sum_increase:.4f}, count_inc={count_increase:.1f})")
        else:
            skipped_no_increase += 1
            status = labels_key[1] if len(labels_key) > 1 else "unknown"
            print(f"    Skipping {method} {status}: count_increase={count_increase}")

    print(f"\n  Summary: matched={matched}, skipped_no_match={skipped_no_match}, skipped_too_few_points={skipped_too_few_points}, skipped_no_increase={skipped_no_increase}")

    request_stats = {
        'total': total_requests,
        'failed': failed_requests,
        'success': success_requests
    }

    return dict(method_durations), request_stats


def fetch_prometheus_histogram_metrics(grafana_url, api_token, base_metric, start_time, end_time):
    """
    Fetches histogram metrics (_sum and _count) and calculates durations.

    Args:
        grafana_url (str): The base URL of the Grafana instance.
        api_token (str): The API token for authentication.
        base_metric (str): The base metric name (without _sum or _count suffix).
        start_time (datetime): The start of the time range.
        end_time (datetime): The end of the time range.

    Returns:
        tuple: (method_durations, request_stats)
            - method_durations: Dictionary with method as key and list of durations as values
            - request_stats: Dictionary with 'total', 'failed', 'success' counts
    """
    print(f"Fetching metrics from {grafana_url}/api/ds/query")
    print(f"Base metric: {base_metric}")
    print(f"Time range: {start_time.isoformat()} to {end_time.isoformat()}")

    print(f"\nFetching {base_metric}_sum...")
    sum_data = fetch_raw_counter_metric(grafana_url, api_token, f"{base_metric}_sum", start_time, end_time)
    print(f"  Found {len(sum_data)} time series")

    print(f"Fetching {base_metric}_count...")
    count_data = fetch_raw_counter_metric(grafana_url, api_token, f"{base_metric}_count", start_time, end_time)
    print(f"  Found {len(count_data)} time series")

    print("\nCalculating durations from histogram data...")
    method_durations, request_stats = calculate_durations_from_histograms(sum_data, count_data)

    for method, durations in method_durations.items():
        if durations:
            print(f"  {method}: {len(durations)} requests, avg: {np.mean(durations):.2f}ms")

    return method_durations, request_stats


def calculate_metrics(durations):
    """
    Calculates trend metrics for a list of durations.

    Args:
        durations (list): A list of numerical durations.

    Returns:
        dict: A dictionary of calculated metrics (avg, max, med, min).
        Note: p(90) and p(95) are omitted due to limitations of histogram data.
    """
    if not durations:
        return {"avg": 0, "max": 0, "med": 0, "min": 0}

    np_durations = np.array(durations)
    return {
        "avg": round(np.mean(np_durations), 2),
        "max": round(np.max(np_durations), 2),
        "med": round(np.median(np_durations), 2),
        "min": round(np.min(np_durations), 2),
    }


def generate_k6_report(method_durations, request_stats, start_time, end_time):
    """
    Generates a k6-like JSON summary report.

    Args:
        method_durations (dict): Dictionary mapping methods to duration lists.
        request_stats (dict): Dictionary with 'total', 'failed', 'success' counts.
        start_time (datetime): The start time of the query range.
        end_time (datetime): The end time of the query range.

    Returns:
        dict: The generated k6 summary report.
    """
    # Combine all durations
    all_durations = []
    for durations in method_durations.values():
        all_durations.extend(durations)

    total_count = request_stats.get('total', 0)
    failed_count = request_stats.get('failed', 0)
    success_count = request_stats.get('success', 0)

    total_duration_seconds = (end_time - start_time).total_seconds()
    rate = total_count / total_duration_seconds if total_duration_seconds > 0 else 0

    # Calculate failure rate (k6's http_req_failed tracks failures, not successes)
    failure_rate = failed_count / total_count if total_count > 0 else 0

    metrics = {
        "http_req_duration": {
            "type": "trend", "contains": "time", "values": calculate_metrics(all_durations)
        },
        "http_reqs": {
            "type": "counter",
            "contains": "default",
            "values": {"count": total_count, "rate": round(rate, 2)},
        },
        "http_req_failed": {
            "type": "rate",
            "contains": "default",
            "values": {
                "passes": success_count,
                "fails": failed_count,
                "value": round(failure_rate, 4)
            },
        },
        "iterations": {
            "type": "counter",
            "contains": "default",
            "values": {"count": total_count, "rate": round(rate, 2)},
        },
    }

    # Add per-method metrics
    for method, durations in method_durations.items():
        if durations:  # Only add metrics for methods that have data
            metrics[f"http_req_duration{{method:{method}}}"] = {
                "type": "trend", "contains": "time", "values": calculate_metrics(durations)
            }

    report = {
        "_note": "Metrics calculated from Prometheus histogram data. Median are synthetic (averaged per endpoint/status).",
        "root_group": {
            "name": "",
            "path": "",
            "id": "d41d8cd98f00b204e9800998ecf8427e",
            "groups": [],
            "checks": [],
        },
        "metrics": metrics
    }

    return report


def main():
    """
    Main function to orchestrate fetching metrics, parsing them, and generating the report.
    """
    parser = argparse.ArgumentParser(description="Generate k6 summary report from Grafana Prometheus metrics.")
    parser.add_argument("--grafana-url", required=True, help="Grafana instance URL.")
    parser.add_argument("--api-token", required=True, help="Grafana API token for authentication.")
    parser.add_argument("--start-time", required=True, help="Start time for the query (ISO 8601 format).")
    parser.add_argument("--end-time", required=True, help="End time for the query (ISO 8601 format).")
    parser.add_argument("--output-file", default="k6-summary.json", help="Output file for the k6 report (default: k6-summary.json)")
    parser.add_argument("--metric", default="nginx_ingress_controller_request_duration_seconds",
                        help="Base metric name without _sum or _count suffix (default: nginx_ingress_controller_request_duration_seconds)")
    args = parser.parse_args()

    api_token = args.api_token

    try:
        # Support both ' ' and 'T' as separators in ISO 8601 datetime strings
        start_time_str = args.start_time.strip().replace(" ", "T")
        end_time_str = args.end_time.strip().replace(" ", "T")
        start_time = datetime.fromisoformat(start_time_str).replace(tzinfo=timezone.utc)
        end_time = datetime.fromisoformat(end_time_str).replace(tzinfo=timezone.utc)
    except ValueError as e:
        print(f"Error: Invalid date format for --start-time or --end-time. Please use ISO 8601 format. Details: {e}")
        exit(1)

    method_durations, request_stats = fetch_prometheus_histogram_metrics(
        args.grafana_url, api_token, args.metric, start_time, end_time
    )

    if not any(method_durations.values()):
        print("\nNo metric data found for the given time range and query.")
        print("This could mean:")
        print("  1. No traffic during this time period")
        print("  2. The metric name is incorrect (try checking available metrics in Grafana)")
        print("  3. The label filters don't match any data")
        print("\nGenerating empty report...")
        empty_stats = {'total': 0, 'failed': 0, 'success': 0}
        report = generate_k6_report({}, empty_stats, start_time, end_time)
    else:
        total_requests = request_stats.get('total', 0)
        failed_requests = request_stats.get('failed', 0)
        success_requests = request_stats.get('success', 0)
        failure_rate = (failed_requests / total_requests * 100) if total_requests > 0 else 0

        print(f"\n{'='*60}")
        print(f"Total requests: {total_requests}")
        print(f"  Success (2xx/3xx): {success_requests}")
        print(f"  Failed (4xx/5xx): {failed_requests}")
        print(f"  Failure rate: {failure_rate:.2f}%")
        print(f"{'='*60}")
        report = generate_k6_report(method_durations, request_stats, start_time, end_time)

    with open(args.output_file, "w") as f:
        json.dump(report, f, indent=4)

    print(f"\nk6 summary report successfully generated at {args.output_file}")
    print("\nIMPORTANT LIMITATIONS:")
    print("  - Percentiles (p90, p95) are omitted due to histogram data limitations")
    print("  - Median/max values may be equal or very close due to aggregated data")
    print("  - Durations are calculated as averages per endpoint/status, not individual requests")
    print("\nReported metrics: avg, min, max, median (synthetic), count, rate")


if __name__ == "__main__":
    main()
