import 'package:flutter/material.dart';

import '../../../domain/model/help/topic_block.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/button/link_button.dart';

class TopicReferences extends StatelessWidget {
  final TopicReferenceBlock block;
  final Function(String)? onReferenceTap;

  const TopicReferences({
    required this.block,
    required this.onReferenceTap,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Divider(),
        const SizedBox(height: 24),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: kDefaultHorizontalPadding),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Semantics(
                header: true,
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    Icon(
                      Icons.help_outline_rounded,
                      size: 20,
                      color: context.colorScheme.onSurface,
                      semanticLabel: '',
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        context.l10n.helpTopicScreenReferencesHeading,
                        style: context.textTheme.titleSmall,
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 8),
              for (final link in block.links)
                Semantics(
                  link: true,
                  child: LinkButton(
                    text: Text(link.label),
                    onPressed: onReferenceTap == null ? null : () => onReferenceTap!(link.topicId),
                  ),
                ),
            ],
          ),
        ),
      ],
    );
  }
}
