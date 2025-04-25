import 'package:wallet_core/core.dart';

import '../../../domain/model/app_image_data.dart';
import '../mapper.dart';

class ImageMapper extends Mapper<Image, AppImageData> {
  ImageMapper();

  @override
  AppImageData map(Image input) {
    return switch (input) {
      Image_Svg(:final xml) => SvgImage(xml),
      Image_Png(:final data) => AppMemoryImage(data),
      Image_Jpeg(:final data) => AppMemoryImage(data),
      Image_Asset(:final path) => AppAssetImage(path),
    };
  }
}
