import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../domain/model/policy/policy.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/duration_extension.dart';
import 'model/policy_entry.dart';

/// Helper class to organize all the provided policy attributes into a render-able list of [PolicyEntry]s
class PolicyEntriesBuilder {
  final BuildContext context;
  final TextStyle urlTheme;

  PolicyEntriesBuilder(this.context, this.urlTheme);

  List<PolicyEntry> build(Policy interactionPolicy) {
    final results = <PolicyEntry>[];

    final dataPurpose = interactionPolicy.dataPurpose;
    final storageDuration = interactionPolicy.storageDuration;
    final privacyPolicyUrl = interactionPolicy.privacyPolicyUrl;

    if (dataPurpose != null) {
      results.add(_buildDataPurposeEntry(dataPurpose, interactionPolicy.dataPurposeDescription));
    }
    results.add(_buildDataSharingPolicy(interactionPolicy));
    if (storageDuration != null) {
      results.add(_buildStorageDurationPolicy(storageDuration));
    }
    if (interactionPolicy.dataIsSignature) {
      results.add(_buildSignaturePolicy());
    }
    if (storageDuration != null && storageDuration.inDays > 0) {
      results.add(_buildDeletionPolicy(interactionPolicy.deletionCanBeRequested));
    }
    if (privacyPolicyUrl != null) {
      results.add(_buildPrivacyPolicy(privacyPolicyUrl));
    }

    return results;
  }

  PolicyEntry _buildDataPurposeEntry(String dataPurpose, String? dataPurposeDescription) {
    return PolicyEntry(
      title: TextSpan(text: dataPurpose),
      description: TextSpan(text: dataPurposeDescription ?? context.l10n.policyScreenDataPurposeDescription),
      icon: Icons.task_outlined,
    );
  }

  PolicyEntry _buildStorageDurationPolicy(Duration storageDuration) {
    return PolicyEntry(
      title: TextSpan(
        text: context.l10n.policyScreenDataRetentionDuration(storageDuration.inMonths),
      ),
      description: TextSpan(text: context.l10n.policyScreenDataRetentionDurationDescription(storageDuration.inMonths)),
      icon: Icons.access_time_outlined,
    );
  }

  PolicyEntry _buildDataSharingPolicy(Policy interactionPolicy) {
    return PolicyEntry(
      title: TextSpan(
        text: interactionPolicy.dataIsShared
            ? context.l10n.policyScreenDataWillBeShared
            : context.l10n.policyScreenDataWillNotBeShared,
      ),
      description: TextSpan(
        text: interactionPolicy.dataIsShared
            ? context.l10n.policyScreenDataWillBeSharedDescription
            : context.l10n.policyScreenDataWillNotBeSharedDescription,
      ),
      icon: Icons.share_outlined,
    );
  }

  PolicyEntry _buildSignaturePolicy() {
    return PolicyEntry(
      title: TextSpan(text: context.l10n.policyScreenDataIsSignature),
      description: const TextSpan(text: _kLoremIpsum),
      icon: Icons.security_outlined,
    );
  }

  PolicyEntry _buildDeletionPolicy(bool deletionCanBeRequested) {
    return PolicyEntry(
      title: TextSpan(
        text: deletionCanBeRequested
            ? context.l10n.policyScreenDataCanBeDeleted
            : context.l10n.policyScreenDataCanNotBeDeleted,
      ),
      description: TextSpan(
        text: deletionCanBeRequested
            ? context.l10n.policyScreenDataCanBeDeletedDescription
            : context.l10n.policyScreenDataCanNotBeDeletedDescription,
      ),
      icon: Icons.delete_outline,
    );
  }

  PolicyEntry _buildPrivacyPolicy(String privacyPolicyUrl) {
    final policyCta = context.l10n.policyScreenPolicySectionPolicyCta;
    final fullPolicyDescription = context.l10n.policyScreenPolicySectionText(policyCta);
    final ctaIndex = fullPolicyDescription.indexOf(policyCta);
    final prefix = fullPolicyDescription.substring(0, ctaIndex);
    final suffix = fullPolicyDescription.substring(ctaIndex + policyCta.length, fullPolicyDescription.length);

    final policyEntry = PolicyEntry(
      title: TextSpan(text: context.l10n.policyScreenPolicySectionTitle),
      description: TextSpan(children: [
        TextSpan(text: prefix),
        TextSpan(
          text: policyCta,
          recognizer: TapGestureRecognizer()
            ..onTap = () => launchUrlString(privacyPolicyUrl, mode: LaunchMode.externalApplication),
          style: urlTheme,
        ),
        TextSpan(text: suffix),
      ]),
    );
    return policyEntry;
  }
}

const _kLoremIpsum =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.';
