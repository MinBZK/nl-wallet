import 'package:wallet_core/core.dart';

import '../../../domain/model/app_image_data.dart';
import '../mapper.dart';

class ImageMapper extends Mapper<Image, AppImageData> {
  ImageMapper();

  @override
  AppImageData map(Image input) {
    return switch (input) {
      Image_Jpeg(:final data) => AppMemoryImage(data),
      Image_Png(:final data) => AppMemoryImage(data),
      Image_Bmp(:final data) => AppMemoryImage(data),
      Image_Svg(:final svg) => SvgImage(svg.xml()),
      Image_Asset(:final path) => AppAssetImage(path),
    };
  }
}
