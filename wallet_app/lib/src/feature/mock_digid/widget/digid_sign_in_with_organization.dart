import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/organization/organization_logo.dart';

const _kOrganizationLogoSize = 72.0;

class DigidSignInWithOrganization extends StatelessWidget {
  const DigidSignInWithOrganization({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const OrganizationLogo(
            image: AssetImage('assets/non-free/images/logo_rijksoverheid.png'),
            size: _kOrganizationLogoSize,
          ),
          const SizedBox(height: 16),
          Text(
            context.l10n.mockDigidScreenSignInOrganization,
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}
