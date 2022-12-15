import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../domain/model/policy/interaction_policy.dart';
import 'model/policy_entry.dart';

/// Helper class to organize all the provided policy attributes into a render-able list of [PolicyEntry]s
class PolicyEntriesBuilder {
  final AppLocalizations locale;
  final TextStyle urlTheme;

  PolicyEntriesBuilder(this.locale, this.urlTheme);

  List<PolicyEntry> build(InteractionPolicy interactionPolicy) {
    final results = <PolicyEntry>[];

    final dataPurpose = interactionPolicy.dataPurpose;
    final storageDuration = interactionPolicy.storageDuration;
    final privacyPolicyUrl = interactionPolicy.privacyPolicyUrl;

    if (dataPurpose != null) {
      results.add(_buildDataPurposeEntry(dataPurpose));
    }
    if (storageDuration != null) {
      results.add(_buildStorageDurationPolicy(storageDuration));
    }
    results.add(_buildDataSharingPolicy(interactionPolicy));
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

  PolicyEntry _buildDataPurposeEntry(String dataPurpose) {
    return PolicyEntry(
      title: TextSpan(text: dataPurpose),
      description: const TextSpan(text: _kLoremIpsum),
      icon: Icons.task_outlined,
    );
  }

  PolicyEntry _buildStorageDurationPolicy(Duration storageDuration) {
    return PolicyEntry(
      title: TextSpan(
        text: locale.policyScreenDataRetentionDuration(
          storageDuration.inDays,
        ),
      ),
      description: const TextSpan(text: _kLoremIpsumShort),
      icon: Icons.access_time_outlined,
    );
  }

  PolicyEntry _buildDataSharingPolicy(InteractionPolicy interactionPolicy) {
    return PolicyEntry(
      title: TextSpan(
        text: interactionPolicy.dataIsShared
            ? locale.policyScreenDataWillBeShared
            : locale.policyScreenDataWillNotBeShared,
      ),
      description: const TextSpan(text: _kLoremIpsum),
      icon: Icons.share_outlined,
    );
  }

  PolicyEntry _buildSignaturePolicy() {
    return PolicyEntry(
      title: TextSpan(text: locale.policyScreenDataIsSignature),
      description: const TextSpan(text: _kLoremIpsum),
      icon: Icons.security_outlined,
    );
  }

  PolicyEntry _buildDeletionPolicy(bool deletionCanBeRequested) {
    return PolicyEntry(
      title: TextSpan(
        text: deletionCanBeRequested ? locale.policyScreenDataCanBeDeleted : locale.policyScreenDataCanNotBeDeleted,
      ),
      description: const TextSpan(text: _kLoremIpsumShort),
      icon: Icons.delete_outline,
    );
  }

  PolicyEntry _buildPrivacyPolicy(String privacyPolicyUrl) {
    final policyCta = locale.policyScreenPolicySectionPolicyCta;
    final fullPolicyDescription = locale.policyScreenPolicySectionText(policyCta);
    final ctaIndex = fullPolicyDescription.indexOf(policyCta);
    final prefix = fullPolicyDescription.substring(0, ctaIndex);
    final suffix = fullPolicyDescription.substring(ctaIndex + policyCta.length, fullPolicyDescription.length);

    final policyEntry = PolicyEntry(
      title: TextSpan(text: locale.policyScreenPolicySectionTitle),
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
const _kLoremIpsumShort =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.';
