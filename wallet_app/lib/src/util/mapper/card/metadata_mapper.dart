import 'dart:ui';

import 'package:wallet_core/core.dart';

import '../../../domain/model/app_image_data.dart';
import '../../../domain/model/card/metadata/card_display_metadata.dart';
import '../../../domain/model/card/metadata/card_rendering.dart';
import '../../extension/locale_extension.dart';
import '../mapper.dart';

class DisplayMetadataMapper extends Mapper<DisplayMetadata, CardDisplayMetadata> {
  final Mapper<Image, AppImageData> _imageMapper;

  DisplayMetadataMapper(this._imageMapper);

  @override
  CardDisplayMetadata map(DisplayMetadata input) {
    final rendering = input.rendering;
    final CardRendering? result = switch (rendering) {
      null => null,
      RenderingMetadata_Simple() => SimpleCardRendering(
          bgColor: rendering.backgroundColor?.toColor(),
          textColor: rendering.textColor?.toColor(),
          logo: rendering.logo?.image == null ? null : _imageMapper.map(rendering.logo!.image),
          logoAltText: rendering.logo?.altText,
        ),
      RenderingMetadata_SvgTemplates() => null,
    };
    return CardDisplayMetadata(
      language: LocaleExtension.parseLocale(input.lang),
      name: input.name,
      description: input.description,
      rawSummary: input.summary,
      rendering: result,
    );
  }
}

extension ColorExtension on String {
  Color? toColor() {
    final hexColor = replaceAll('#', '');
    if (hexColor.length == 6) {
      return Color(int.parse('0xFF$hexColor'));
    } else if (hexColor.length == 8) {
      return Color(int.parse('0x$hexColor'));
    }
    return null;
  }
}
