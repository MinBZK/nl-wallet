import 'package:flutter/material.dart';

const _kImageBorderRadius = 4.0;
const _kImageWidth = 58.0;
const _kImageHeight = 64.0;

class DataAttributeImage extends StatelessWidget {
  final String image;

  const DataAttributeImage({required this.image, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: _kImageWidth,
      height: _kImageHeight,
      child: ClipRRect(
        borderRadius: const BorderRadius.all(Radius.circular(_kImageBorderRadius)),
        child: Image.asset(image),
      ),
    );
  }
}
