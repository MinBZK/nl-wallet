import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

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
            AppLocalizations.of(context).mockDigidScreenSignInOrganization,
            style: Theme.of(context).textTheme.displayMedium,
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}
