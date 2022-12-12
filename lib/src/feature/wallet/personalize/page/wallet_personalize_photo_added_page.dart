import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../common/widget/link_button.dart';
import '../../../common/widget/placeholder_screen.dart';

class WalletPersonalizePhotoAddedPage extends StatelessWidget {
  final DataAttribute photo;
  final VoidCallback onNextPressed;

  const WalletPersonalizePhotoAddedPage({
    required this.onNextPressed,
    Key? key,
    required this.photo,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        mainAxisSize: MainAxisSize.max,
        children: [
          ClipRRect(
            borderRadius: BorderRadius.circular(12),
            child: Image.asset(
              photo.value,
              width: 188,
              height: 208,
              fit: BoxFit.cover,
            ),
          ),
          const SizedBox(height: 32),
          Text(
            locale.walletPersonalizePhotoAddedPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            locale.walletPersonalizePhotoAddedPageSubtitle,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
          const Divider(height: 32),
          LinkButton(
            child: Text(locale.walletPersonalizePhotoAddedPageDataIncorrectCta),
            onPressed: () => PlaceholderScreen.show(context, locale.walletPersonalizePhotoAddedPageDataIncorrectCta),
          ),
          const Divider(height: 32),
          const Spacer(),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: ElevatedButton(
              onPressed: onNextPressed,
              child: Text(locale.walletPersonalizePhotoAddedPageContinueCta),
            ),
          ),
        ],
      ),
    );
  }
}
