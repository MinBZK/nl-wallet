import 'dart:typed_data';

import 'package:equatable/equatable.dart';

import '../../feature/common/widget/app_image.dart';

/// Represents any image that can be rendered using the [AppImage] widget and aligns
/// with the variants of images that are provided by the wallet_core.
sealed class AppImageData extends Equatable {
  const AppImageData();
}

class SvgImage extends AppImageData {
  final String data;

  @override
  List<Object?> get props => [data];

  const SvgImage(this.data);
}

class AppAssetImage extends AppImageData {
  final String name;

  @override
  List<Object?> get props => [name];

  const AppAssetImage(this.name);
}

class AppMemoryImage extends AppImageData {
  final Uint8List data;

  @override
  List<Object?> get props => [data];

  const AppMemoryImage(this.data);
}
