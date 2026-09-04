import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../util/formatter/attribute_value_formatter.dart';
import '../../../../util/helper/bsn_helper.dart';
import '../../../../util/helper/semantics_helper.dart';
import '../app_image.dart';
import '../bullet_list_dot.dart';
import '../list/list_item.dart';

/// Bounds an [ImageValue], e.g. the portrait of an mDL.
const _kMaxImageSize = 160.0;
const _kImageBorderRadius = 4.0;

class DataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    return ListItem(
      label: _buildLabel(context, attribute.value),
      subtitle: _buildSubtitle(context, attribute.value),
    );
  }

  Widget _buildLabel(BuildContext context, AttributeValue attributeValue) {
    InlineSpan labelSpan = attribute.label.l10nSpan(context);
    if (attributeValue is ArrayValue && attributeValue.value.length > 1) {
      final label = attribute.label.l10nValue(context);
      final length = attributeValue.value.length;
      labelSpan = '$label ($length)'.toTextSpan(context);
    }
    return Text.rich(labelSpan);
  }

  Widget _buildSubtitle(BuildContext context, AttributeValue attributeValue) {
    // Check for non-empty array, this value is not simply formatted and displayed.
    if (attributeValue is ArrayValue && attributeValue.value.isNotEmpty) {
      return _buildArrayStyleSubtitle(context, attributeValue);
    }

    // An image is rendered instead of formatted.
    if (attributeValue is ImageValue) return _buildImageSubtitle(context, attributeValue);

    // A map is spread over one row per entry, so that the keys line up.
    if (attributeValue is MapValue && attributeValue.value.isNotEmpty) {
      return _buildMapStyleSubtitle(context, attributeValue);
    }

    final prettyValue = attributeValue.prettyPrint(context);
    return Text.rich(
      prettyValue.toTextSpan(context),
      semanticsLabel: BsnHelper.isValidBsnFormat(prettyValue) ? SemanticsHelper.splitNumberString(prettyValue) : null,
      style: _resolveSubtitleStyle(context, attribute.value),
    );
  }

  Widget _buildImageSubtitle(BuildContext context, ImageValue imageValue) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: Padding(
        padding: const EdgeInsets.only(top: 4),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(_kImageBorderRadius),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: _kMaxImageSize, maxHeight: _kMaxImageSize),
            child: AppImage(
              asset: imageValue.value,
              fit: BoxFit.contain,
              // Without this the row announces the label but never the fact that it holds a value.
              altText: imageValue.prettyPrint(context),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildMapStyleSubtitle(BuildContext context, MapValue mapValue) {
    return Semantics(
      // Dedicated semanticsLabel, this makes sure it gets announced on Android as well.
      attributedLabel: _buildAttributedString(context, mapValue),
      excludeSemantics: true,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: mapValue.value.entries
            .map(
              (entry) => Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text.rich(
                    '${AttributeValueFormatter.formatMapKey(context.activeLocale, entry.key)}: '.toTextSpan(context),
                    style: context.textTheme.bodyMedium,
                  ),
                  Expanded(child: _buildSubtitle(context, entry.value)),
                ],
              ),
            )
            .toList(),
      ),
    );
  }

  Widget _buildArrayStyleSubtitle(BuildContext context, ArrayValue arrayValue) {
    return Semantics(
      // Dedicated semanticsLabel, this makes sure it gets announced on Android as well.
      attributedLabel: _buildAttributedString(context, arrayValue),
      excludeSemantics: true,
      child: ListView.builder(
        shrinkWrap: true,
        physics: const NeverScrollableScrollPhysics(),
        itemBuilder: (c, i) {
          final subtitleRow = _buildSubtitle(context, arrayValue.value[i]);
          return Row(
            // Aligns the dot with the first line, which matters for multi line entries such as a map.
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const SizedBox(width: 24, height: 24, child: BulletListDot()),
              Expanded(child: subtitleRow),
            ],
          );
        },
        itemCount: arrayValue.value.length,
      ),
    );
  }

  AttributedString _buildAttributedString(BuildContext context, AttributeValue attributeValue) =>
      attributeValue.prettyPrint(context, inline: true).toAttributedString(context);

  TextStyle? _resolveSubtitleStyle(BuildContext context, AttributeValue attributeValue) {
    switch (attributeValue) {
      case ArrayValue():
        return attributeValue.value.isEmpty ? context.textTheme.bodyLarge : null;
      case NullValue():
        return context.textTheme.bodyLarge;
      case StringValue():
        return attributeValue.value.isEmpty ? context.textTheme.bodyLarge : null;
      case BooleanValue():
      case NumberValue():
      case DateValue():
      case BytesValue():
      case ImageValue():
      case MapValue():
        return null;
    }
  }
}
