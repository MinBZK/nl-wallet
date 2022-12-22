import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class DigidSignInWithHeader extends StatelessWidget {
  const DigidSignInWithHeader({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.mockDigidScreenHeaderTitle,
            style: Theme.of(context).textTheme.headline2?.copyWith(color: Theme.of(context).primaryColor),
          ),
          const SizedBox(height: 8),
          Text(
            locale.mockDigidScreenHeaderSubtitle,
            style: Theme.of(context).textTheme.bodyText2?.copyWith(fontWeight: FontWeight.bold),
          ),
        ],
      ),
    );
  }
}
