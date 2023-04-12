import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class ShowDetailsCta extends StatelessWidget {
  const ShowDetailsCta({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(AppLocalizations.of(context).showDetailsCta, style: Theme.of(context).textTheme.labelLarge),
        const SizedBox(width: 8),
        Icon(
          Icons.arrow_forward,
          color: Theme.of(context).textTheme.labelLarge?.color,
          size: 16 * MediaQuery.of(context).textScaleFactor,
        ),
      ],
    );
  }
}
