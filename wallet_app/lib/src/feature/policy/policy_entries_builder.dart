import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/organization.dart';
import '../../domain/model/policy/policy.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/duration_extension.dart';
import '../common/widget/url_span.dart';
import 'model/policy_entry.dart';

/// Helper class to organize all the provided policy attributes into a render-able list of [PolicyEntry]s
class PolicyEntriesBuilder {
  final BuildContext context;
  final TextStyle urlTheme;
  final bool addSignatureEntry;

  PolicyEntriesBuilder(this.context, this.urlTheme, {this.addSignatureEntry = false});

  List<PolicyEntry> build(Organization organization, Policy policy) {
    final results = <PolicyEntry>[];

    final dataPurpose = policy.dataPurpose;
    final storageDuration = policy.storageDuration;
    final privacyPolicyUrl = policy.privacyPolicyUrl;

    if (dataPurpose != null) {
      results.add(_buildDataPurposeEntry(dataPurpose, policy.dataPurposeDescription));
    }
    results.add(_buildDataSharingPolicy(policy));
    if (storageDuration != null) {
      results.add(_buildStorageDurationPolicy(storageDuration));
    } else {
      results.add(_buildDataNotStoredPolicy());
    }
    if (addSignatureEntry) {
      results.add(_buildSignaturePolicy());
    }
    if (storageDuration != null && storageDuration.inDays > 0) {
      results.add(_buildDeletionPolicy(policy.deletionCanBeRequested));
    }
    if (privacyPolicyUrl != null) {
      results.add(_buildPrivacyPolicy(organization.displayName.l10nValue(context), privacyPolicyUrl));
    }

    return results;
  }

  PolicyEntry _buildDataPurposeEntry(String dataPurpose, String? dataPurposeDescription) {
    return PolicyEntry(
      title: TextSpan(
        text: dataPurpose,
        locale: context.activeLocale,
      ),
      description: TextSpan(
        text: dataPurposeDescription ?? context.l10n.policyScreenDataPurposeDescription,
        locale: context.activeLocale,
      ),
      icon: Icons.task_outlined,
    );
  }

  PolicyEntry _buildStorageDurationPolicy(Duration storageDuration) {
    return PolicyEntry(
      title: TextSpan(
        text: context.l10n.policyScreenDataRetentionDuration(storageDuration.inMonths),
        locale: context.activeLocale,
      ),
      description: TextSpan(
        text: context.l10n.policyScreenDataRetentionDurationDescription(storageDuration.inMonths),
        locale: context.activeLocale,
      ),
      icon: Icons.access_time_outlined,
    );
  }

  PolicyEntry _buildDataNotStoredPolicy() {
    return PolicyEntry(
      title: TextSpan(
        text: context.l10n.policyScreenDataNotBeStored,
        locale: context.activeLocale,
      ),
      description: TextSpan(
        text: context.l10n.policyScreenDataNotBeStoredDescription,
        locale: context.activeLocale,
      ),
      icon: Icons.access_time_outlined,
    );
  }

  PolicyEntry _buildDataSharingPolicy(Policy interactionPolicy) {
    return PolicyEntry(
      title: TextSpan(
        text: interactionPolicy.dataIsShared
            ? context.l10n.policyScreenDataWillBeShared
            : context.l10n.policyScreenDataWillNotBeShared,
        locale: context.activeLocale,
      ),
      description: TextSpan(
        text: interactionPolicy.dataIsShared
            ? context.l10n.policyScreenDataWillBeSharedDescription
            : context.l10n.policyScreenDataWillNotBeSharedDescription,
        locale: context.activeLocale,
      ),
      icon: Icons.share_outlined,
    );
  }

  PolicyEntry _buildSignaturePolicy() {
    return PolicyEntry(
      title: TextSpan(
        text: context.l10n.policyScreenDataIsSignature,
        locale: context.activeLocale,
      ),
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
        locale: context.activeLocale,
      ),
      description: TextSpan(
        text: deletionCanBeRequested
            ? context.l10n.policyScreenDataCanBeDeletedDescription
            : context.l10n.policyScreenDataCanNotBeDeletedDescription,
        locale: context.activeLocale,
      ),
      icon: Icons.delete_outline,
    );
  }

  PolicyEntry _buildPrivacyPolicy(String organizationName, String privacyPolicyUrl) {
    final policyCta = context.l10n.policyScreenPolicySectionPolicyCta;
    final fullPolicyDescription = context.l10n.policyScreenPolicySectionText(organizationName, policyCta);
    final ctaIndex = fullPolicyDescription.indexOf(policyCta);
    final prefix = fullPolicyDescription.substring(0, ctaIndex);
    final suffix = fullPolicyDescription.substring(ctaIndex + policyCta.length, fullPolicyDescription.length);

    final policyEntry = PolicyEntry(
      title: TextSpan(
        text: context.l10n.policyScreenPolicySectionTitle,
        locale: context.activeLocale,
      ),
      description: TextSpan(
        locale: context.activeLocale,
        children: [
          TextSpan(text: prefix),
          UrlSpan(
            ctaText: policyCta,
            onPressed: () => launchUrlString(privacyPolicyUrl, mode: LaunchMode.externalApplication),
          ),
          TextSpan(text: suffix),
        ],
      ),
      descriptionSemanticsLabel: prefix + policyCta + suffix,
      semanticOnTap: () => launchUrlString(privacyPolicyUrl, mode: LaunchMode.externalApplication),
      semanticOnTapHint: context.l10n.generalWCAGOpenLink,
    );
    return policyEntry;
  }
}

const _kLoremIpsum =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.';
