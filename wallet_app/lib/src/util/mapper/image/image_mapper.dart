import 'package:wallet_core/core.dart';

import '../../../domain/model/app_image_data.dart';
import '../mapper.dart';

class ImageMapper extends Mapper<Image, AppImageData> {
  ImageMapper();

  @override
  AppImageData map(Image input) => input.map(
        svg: (svg) => SvgImage(svg.xml),
        png: (png) => Base64Image(png.base64),
        jpg: (jpg) => Base64Image(jpg.base64),
      );
}
