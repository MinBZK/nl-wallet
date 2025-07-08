import 'package:flutter/material.dart';

import '../../domain/model/organization.dart';
import '../../domain/model/policy/policy.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/duration_extension.dart';
import '../common/widget/list/list_item.dart';

/// Helper class to organize all the provided policy attributes into list of [ListItem] widgets.
class PolicyRowBuilder {
  final BuildContext context;
  final bool addSignatureEntry;

  PolicyRowBuilder(this.context, {this.addSignatureEntry = false});

  List<Widget> build(Organization organization, Policy policy) {
    final results = <Widget>[];

    final dataPurpose = policy.dataPurpose;
    final storageDuration = policy.storageDuration;

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
    return results;
  }

  Widget _buildPolicyRow(
    BuildContext context, {
    required String title,
    required String description,
    required IconData icon,
  }) {
    return ListItem.horizontal(
      label: Semantics(header: true, child: Text(title)),
      subtitle: Text(description),
      icon: Icon(icon),
    );
  }

  Widget _buildDataPurposeEntry(String dataPurpose, String? dataPurposeDescription) {
    return _buildPolicyRow(
      context,
      title: dataPurpose,
      description: dataPurposeDescription ?? context.l10n.policyScreenDataPurposeDescription,
      icon: Icons.task_outlined,
    );
  }

  Widget _buildStorageDurationPolicy(Duration storageDuration) {
    return _buildPolicyRow(
      context,
      title: context.l10n.policyScreenDataRetentionDuration(storageDuration.inMonths),
      description: context.l10n.policyScreenDataRetentionDurationDescription(storageDuration.inMonths),
      icon: Icons.access_time_outlined,
    );
  }

  Widget _buildDataNotStoredPolicy() {
    return _buildPolicyRow(
      context,
      title: context.l10n.policyScreenDataNotBeStored,
      description: context.l10n.policyScreenDataNotBeStoredDescription,
      icon: Icons.access_time_outlined,
    );
  }

  Widget _buildDataSharingPolicy(Policy interactionPolicy) {
    return _buildPolicyRow(
      context,
      title: interactionPolicy.dataIsShared
          ? context.l10n.policyScreenDataWillBeShared
          : context.l10n.policyScreenDataWillNotBeShared,
      description: interactionPolicy.dataIsShared
          ? context.l10n.policyScreenDataWillBeSharedDescription
          : context.l10n.policyScreenDataWillNotBeSharedDescription,
      icon: Icons.share_outlined,
    );
  }

  Widget _buildSignaturePolicy() {
    return _buildPolicyRow(
      context,
      title: context.l10n.policyScreenDataIsSignature,
      description: _kLoremIpsum,
      icon: Icons.security_outlined,
    );
  }

  Widget _buildDeletionPolicy(bool deletionCanBeRequested) {
    return _buildPolicyRow(
      context,
      title: deletionCanBeRequested
          ? context.l10n.policyScreenDataCanBeDeleted
          : context.l10n.policyScreenDataCanNotBeDeleted,
      description: deletionCanBeRequested
          ? context.l10n.policyScreenDataCanBeDeletedDescription
          : context.l10n.policyScreenDataCanNotBeDeletedDescription,
      icon: Icons.delete_outline,
    );
  }
}

const _kLoremIpsum =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.';
