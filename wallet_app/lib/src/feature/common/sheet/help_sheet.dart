import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../screen/placeholder_screen.dart';
import '../widget/button/link_button.dart';
import '../widget/button/text_icon_button.dart';
import '../widget/os_version_text.dart';
import '../widget/version_text.dart';

class HelpSheet extends StatelessWidget {
  final String? errorCode, supportCode;

  const HelpSheet({
    this.errorCode,
    this.supportCode,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        MergeSemantics(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: Text(
                  context.l10n.helpSheetTitle,
                  style: context.textTheme.displayMedium,
                  textAlign: TextAlign.start,
                ),
              ),
              const SizedBox(height: 16),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: Text(
                  context.l10n.helpSheetDescription,
                  style: context.textTheme.bodyLarge,
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: _buildInfoSection(context),
        ),
        const Divider(height: 32),
        LinkButton(
          onPressed: () => PlaceholderScreen.show(context),
          customPadding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(context.l10n.helpSheetHelpdeskCta),
        ),
        const Divider(height: 32),
        Center(
          child: TextIconButton(
            icon: Icons.close,
            iconPosition: IconPosition.start,
            child: Text(context.l10n.helpSheetCloseCta),
            onPressed: () => Navigator.pop(context),
          ),
        ),
      ],
    );
  }

  Widget _buildInfoSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        VersionText(
          textStyle: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
        ),
        OsVersionText(
          textStyle: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
        ),
        errorCode == null
            ? const SizedBox.shrink()
            : Text(
                context.l10n.helpSheetErrorCode(errorCode!),
                style: context.textTheme.bodyMedium
                    ?.copyWith(fontWeight: FontWeight.bold, color: context.colorScheme.error),
              ),
        supportCode == null
            ? const SizedBox.shrink()
            : Text(
                context.l10n.helpSheetSupportCode(supportCode!),
                style: context.textTheme.bodyMedium
                    ?.copyWith(fontWeight: FontWeight.bold, color: context.colorScheme.error),
              )
      ],
    );
  }

  static Future<void> show(
    BuildContext context, {
    String? errorCode,
    String? supportCode,
  }) async {
    return showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      builder: (BuildContext context) {
        return DraggableScrollableSheet(
          expand: false,
          builder: (context, scrollController) => SingleChildScrollView(
            controller: scrollController,
            child: HelpSheet(
              errorCode: errorCode,
              supportCode: supportCode,
            ),
          ),
        );
      },
    );
  }
}
