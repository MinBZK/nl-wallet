#!/usr/bin/env python3

import os
import re
import subprocess
import sys
from datetime import datetime, timedelta

import gitlab
from jira import JIRA
from jira.exceptions import JIRAError



def get_env_var(key, required=True, default=None):
    val = os.getenv(key, default)
    if required and val is None:
        sys.exit(f"Error: Missing required environment variable: {key}")
    return val


def run_shell(cmd_args):
    try:
        output = subprocess.check_output(cmd_args, text=True)
        return output.strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"Command failed: {' '.join(cmd_args)}\n{e.stderr or e}")


def fetch_git_tags():
    subprocess.run(["git", "fetch", "--tags"], check=True)
    tags = run_shell(["git", "tag", "--sort=-creatordate"]).splitlines()
    return [t for t in tags if re.match(r"^v\d+\.\d+\.\d+$", t)]


def get_tag_date(tag):
    return int(run_shell(["git", "log", "-1", f"--format=%ct", tag]))


def extract_jira_key(branch):
    match = re.match(r"[A-Z]+-\d+", branch.upper())
    return match.group(0) if match else None


def init_gitlab_client():
    url = get_env_var("CI_SERVER_URL")
    token = get_env_var("GITLAB_TOKEN")
    return gitlab.Gitlab(url, private_token=token)


class JiraClientWrapper:
    def __init__(self):
        jira_url = get_env_var("JIRA_BASE_URL")
        jira_token = get_env_var("JIRA_PAT")
        self.client = JIRA(jira_url, token_auth=jira_token)

    def fetch_issue(self, key):
        return self.client.issue(key, fields="fixVersions,parent")

    def fetch_issues_for_version(self, version):
        jql = f'fixVersion="{version}"'
        return self.client.search_issues(jql, fields="fixVersions,parent")

    def update_fix_version(self, issue_key, version):
        issue = self.client.issue(issue_key)
        issue.update(fields={"fixVersions": [{"name": version}]})


def fetch_merged_mrs(gl, project_id, target_branch, since):
    project = gl.projects.get(project_id)
    return project.mergerequests.list(
        state='merged',
        target_branch=target_branch,
        updated_after=since,
        all=True
    )


def verify_release():
    tags = fetch_git_tags()
    if len(tags) < 2:
        sys.exit("Error: Not enough release tags.")
    current_tag, previous_tag = tags[:2]
    current_ts, previous_ts = list(map(get_tag_date, [current_tag, previous_tag]))
    if previous_ts > current_ts:
        current_ts, previous_ts = previous_ts, current_ts

    print(f"Release window: [{previous_ts} → {current_ts}] ({previous_tag} → {current_tag})")

    gl = init_gitlab_client()
    jira = JiraClientWrapper()

    project_id = get_env_var("CI_PROJECT_ID")
    target_branch = get_env_var("TARGET_BRANCH", default=get_env_var("CI_DEFAULT_BRANCH"))

    since = (datetime.now() - timedelta(days=365)).isoformat()
    mrs = fetch_merged_mrs(gl, project_id, target_branch, since)

    mr_keys = {}
    for mr in mrs:
        merged_at = datetime.strptime(mr.merged_at, "%Y-%m-%dT%H:%M:%S.%fZ").timestamp()
        branch = mr.source_branch
        if previous_ts <= merged_at <= current_ts:
            key = extract_jira_key(branch)
            if key:
                mr_keys[mr.iid] = key
            else:
                warning_msg = f"[WARN] MR {mr.iid} (branch: {branch}) in timeframe has no Jira key"
                print(warning_msg)
                with open("mrs_missing_jira.log", "a") as log_file:
                    log_file.write(warning_msg + "\n")
        else:
            print(f"[DEBUG] Skipping MR {mr.iid} merged_at {mr.merged_at} (outside release window)")

    print(f"[DEBUG] Extracted Jira keys from MRs in window: {list(mr_keys.values())}")

    jira_issues = jira.fetch_issues_for_version(current_tag)
    print(f"[DEBUG] JIRA issues returned for fixVersion={current_tag}: {[issue.key for issue in jira_issues]}")
    fixed_keys = {issue.key for issue in jira_issues}
    children = {
        issue.fields.parent.key: issue.key
        for issue in jira_issues
        if getattr(issue.fields, "parent", None)
    }

    for key in fixed_keys:
        if not any(j == key or children.get(j) == key for j in mr_keys.values()):
            print(f"[WARN] Jira issue {key} has fixVersion={current_tag} but no MR in release window")

    for mr_id, jira_key in mr_keys.items():
        try:
            issue = jira.fetch_issue(jira_key)
        except JIRAError as e:
            warning_msg = (
                f"[WARN] MR {mr_id} refers to {jira_key} which was not found in Jira. "
                f"message: {getattr(e, 'text', str(e))}"
            )
            
            print(warning_msg)
            
            with open("mrs_missing_jira.log", "a") as log_file:
                log_file.write(warning_msg + "\n")
            
            continue

        versions = [v.name for v in issue.fields.fixVersions]
        parent = getattr(issue.fields, "parent", None)
        parent_key = parent.key if parent else None

        if len(versions) > 1:
            print(f"[WARN] Jira issue {jira_key} has multiple fixVersions: {versions}")
        elif versions and current_tag not in versions:
            print(f"[WARN] Jira issue {jira_key} has incorrect fixVersion(s): {versions}")
        elif not versions:
            print(f"[WARN] Jira issue {jira_key} has no fixVersion")
        elif current_tag in versions:
            continue
        if parent_key:
            try:
                parent = jira.fetch_issue(parent_key)
                parent_versions = [v.name for v in parent.fields.fixVersions]
                if len(parent_versions) > 1:
                    print(f"[WARN] Parent issue {parent_key} has multiple fixVersions: {parent_versions}")
                elif parent_versions and current_tag not in parent_versions:
                    print(f"[WARN] Parent issue {parent_key} has incorrect fixVersion(s): {parent_versions}")
                elif not parent_versions:
                    print(f"[WARN] Parent issue {parent_key} has no fixVersion")
                elif current_tag in parent_versions:
                    continue
            except JIRAError as e:
                print(
                    f"[WARN] Failed to fetch parent issue {parent_key} from Jira. message: {getattr(e, 'text', str(e))}")
        print(f"[WARN] MR {mr_id} refers to {jira_key} which lacks fixVersion = {current_tag}")

    print("✅ Release verification complete.")


def sync_nightly():
    gl = init_gitlab_client()
    jira = JiraClientWrapper()

    project_id = get_env_var("CI_PROJECT_ID")
    target_branch = get_env_var("TARGET_BRANCH", default=get_env_var("CI_DEFAULT_BRANCH"))
    current_release = get_env_var("CURRENT_RELEASE")

    tags = fetch_git_tags()
    if not tags:
        print("No release tags found.")
        return
    last_tag = tags[0]
    last_tag_ts = get_tag_date(last_tag)
    last_tag_dt = datetime.utcfromtimestamp(last_tag_ts).isoformat()

    mrs = fetch_merged_mrs(gl, project_id, target_branch, last_tag_dt)
    print(f"Found {len(mrs)} MRs merged since last release tag: {last_tag}")

    seen_jira_keys = set()

    for mr in mrs:
        mr_id = mr.iid
        branch = getattr(mr, "source_branch", "")

        jira_key = extract_jira_key(branch)
        if not jira_key:
            print(f"[WARN] MR {mr_id} (branch: {branch}) — no Jira key found")
            continue

        seen_jira_keys.add(jira_key)

        try:
            issue = jira.fetch_issue(jira_key)
        except JIRAError as e:
            print(f"[WARN] MR {mr_id} — Jira issue {jira_key} not found. message: {getattr(e, 'text', str(e))}")
            continue

        parent = getattr(issue.fields, "parent", None)
        parent_key = parent.key if parent else None
        target_key = parent_key or jira_key

        try:
            target_issue = jira.fetch_issue(target_key)
            versions = [v.name for v in target_issue.fields.fixVersions]

            if len(versions) > 1:
                print(f"[WARN] Jira issue {target_key} has multiple fixVersions: {versions}. Overwriting with {current_release}")
                jira.update_fix_version(target_key, current_release)
                print(f"[DONE] fixVersion set on {target_key}")
            elif versions and versions[0] != current_release:
                print(f"[WARN] Jira issue {target_key} has incorrect fixVersion: {versions[0]}. Overwriting with {current_release}")
                jira.update_fix_version(target_key, current_release)
                print(f"[DONE] fixVersion set on {target_key}")
            elif not versions:
                print(f"[ACTION] Setting fixVersion {current_release} on {target_key}")
                jira.update_fix_version(target_key, current_release)
                print(f"[DONE] fixVersion set on {target_key}")
            else:
                print(f"[OK] {target_key} already has fixVersion {current_release}")
        except Exception as e:
            print(f"[ERROR] Failed to update fixVersion on {target_key}: {e}")

    print("✅ nightly_git_jira_check complete.")


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in ["verify-release", "sync-nightly"]:
        print("Usage: python script.py [verify-release|sync-nightly]")
        sys.exit(1)

    command = sys.argv[1]
    if command == "verify-release":
        verify_release()
    elif command == "sync-nightly":
        sync_nightly()


if __name__ == "__main__":
    main()
