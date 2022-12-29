import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/document.dart';
import '../../verification/model/organization.dart';
import 'link_button.dart';
import 'placeholder_screen.dart';

class DocumentSection extends StatelessWidget {
  final Document document;
  final Organization organization;
  final EdgeInsets padding;

  const DocumentSection({
    required this.document,
    required this.organization,
    this.padding = EdgeInsets.zero,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: padding,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            document.title,
            style: Theme.of(context).textTheme.subtitle1,
            textAlign: TextAlign.start,
          ),
          Text(
            organization.name,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
          Text(
            document.fileName,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          LinkButton(
            customPadding: EdgeInsets.zero,
            child: Text(AppLocalizations.of(context).checkAgreementPageShowDocumentCta),
            onPressed: () => PlaceholderScreen.show(context, type: PlaceholderType.contract),
          ),
        ],
      ),
    );
  }
}
