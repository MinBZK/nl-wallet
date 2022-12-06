import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../verification/model/verifier_policy.dart';
import 'model/policy_entry.dart';

/// Helper class to organize all the provided policy attributes into a render-able list of [PolicyEntry]s
class PolicyEntriesBuilder {
  final AppLocalizations locale;
  final TextStyle urlTheme;

  PolicyEntriesBuilder(this.locale, this.urlTheme);

  List<PolicyEntry> build(VerifierPolicy policy) {
    final result = <PolicyEntry>[];
    result.add(_buildDataPurposeEntry(policy));
    result.add(_buildStorageDurationPolicy(policy));
    result.add(_buildDataSharingPolicy(policy));
    if (policy.storageDuration.inDays > 0) {
      result.add(_buildDeletionPolicy(policy));
    }
    result.add(_buildPrivacyPolicy(policy));
    return result;
  }

  PolicyEntry _buildDataPurposeEntry(VerifierPolicy policy) {
    return PolicyEntry(
      title: TextSpan(text: policy.dataPurpose),
      description: const TextSpan(text: _kLoremIpsum),
      icon: Icons.task_outlined,
    );
  }

  PolicyEntry _buildStorageDurationPolicy(VerifierPolicy policy) {
    return PolicyEntry(
      title: TextSpan(
        text: locale.verifierPolicyScreenDataRetentionDuration(
          policy.storageDuration.inDays,
        ),
      ),
      description: const TextSpan(text: _kLoremIpsumShort),
      icon: Icons.schedule,
    );
  }

  PolicyEntry _buildDataSharingPolicy(VerifierPolicy policy) {
    return PolicyEntry(
      title: TextSpan(
        text: policy.dataIsShared
            ? locale.verifierPolicyScreenDataWillBeShared
            : locale.verifierPolicyScreenDataWillNotBeShared,
      ),
      description: const TextSpan(text: _kLoremIpsum),
      icon: Icons.share_outlined,
    );
  }

  PolicyEntry _buildDeletionPolicy(VerifierPolicy policy) {
    return PolicyEntry(
      title: TextSpan(
        text: policy.deletionCanBeRequested
            ? locale.verifierPolicyScreenDataCanBeDeleted
            : locale.verifierPolicyScreenDataCanNotBeDeleted,
      ),
      description: const TextSpan(text: _kLoremIpsumShort),
      icon: Icons.delete_outline,
    );
  }

  PolicyEntry _buildPrivacyPolicy(VerifierPolicy policy) {
    final policyCta = locale.verifierPolicyScreenPolicySectionPolicyCta;
    final fullPolicyDescription = locale.verifierPolicyScreenPolicySectionText(policyCta);
    final ctaIndex = fullPolicyDescription.indexOf(policyCta);
    final prefix = fullPolicyDescription.substring(0, ctaIndex);
    final suffix = fullPolicyDescription.substring(ctaIndex + policyCta.length, fullPolicyDescription.length);

    final policyEntry = PolicyEntry(
      title: TextSpan(text: locale.verifierPolicyScreenPolicySectionTitle),
      description: TextSpan(children: [
        TextSpan(text: prefix),
        TextSpan(
          text: policyCta,
          recognizer: TapGestureRecognizer()
            ..onTap = () => launchUrlString(policy.privacyPolicyUrl, mode: LaunchMode.externalApplication),
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
const _kLoremIpsumShort =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.';
