import 'package:flutter/material.dart';

import '../../domain/model/organization.dart';
import '../../domain/model/policy/policy.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/list/list_item.dart';

/// Helper class to organize all the provided policy attributes into list of [ListItem] widgets.
class PolicyRowBuilder {
  final BuildContext context;
  final bool addSignatureEntry;

  PolicyRowBuilder(this.context, {this.addSignatureEntry = false});

  List<Widget> build(Organization organization, Policy policy) {
    final results = <Widget>[];

    final dataPurpose = policy.dataPurpose;

    if (dataPurpose != null) {
      results.add(_buildDataPurposeEntry(dataPurpose, policy.dataPurposeDescription));
    }
    if (addSignatureEntry) {
      results.add(_buildSignaturePolicy());
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
      label: Semantics(header: true, headingLevel: 1, child: Text(title)),
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

  Widget _buildSignaturePolicy() {
    return _buildPolicyRow(
      context,
      title: context.l10n.policyScreenDataIsSignature,
      description: _kLoremIpsum,
      icon: Icons.security_outlined,
    );
  }
}

const _kLoremIpsum =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.';
