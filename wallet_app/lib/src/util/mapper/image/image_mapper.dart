import 'package:wallet_core/core.dart';

import '../../../domain/model/app_image_data.dart';
import '../mapper.dart';

class ImageMapper extends Mapper<Image, AppImageData> {
  ImageMapper();

  @override
  AppImageData map(Image input) {
    return switch (input) {
      Image_Svg(:final xml) => SvgImage(xml),
      Image_Png(:final base64) => Base64Image(base64),
      Image_Jpg(:final base64) => Base64Image(base64),
      Image_Asset(:final path) => AppAssetImage(path),
    };
  }
}
