import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/document.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../screen/placeholder_screen.dart';
import 'button/link_button.dart';

class DocumentSection extends StatelessWidget {
  final Document document;
  final Organization organization;
  final EdgeInsets padding;

  const DocumentSection({
    required this.document,
    required this.organization,
    this.padding = EdgeInsets.zero,
    super.key,
  });

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
            style: context.textTheme.titleMedium,
            textAlign: TextAlign.start,
          ),
          Text(
            organization.displayName.l10nValue(context),
            style: context.textTheme.bodyLarge,
            textAlign: TextAlign.start,
          ),
          Text(
            document.fileName,
            style: context.textTheme.bodyLarge,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          LinkButton(
            text: Text.rich(context.l10n.checkAgreementPageShowDocumentCta.toTextSpan(context)),
            onPressed: () => PlaceholderScreen.showContract(context),
          ),
        ],
      ),
    );
  }
}
