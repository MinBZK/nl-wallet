import 'package:flutter/material.dart';

const _kImageBorderRadius = 4.0;
const _kImageWidth = 58.0;
const _kImageHeight = 64.0;

class DataAttributeRowImage extends StatelessWidget {
  final ImageProvider image;
  final String? label;

  const DataAttributeRowImage({required this.image, this.label, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final labelVisible = label?.isNotEmpty ?? false;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Visibility(
          visible: labelVisible,
          child: Padding(
            padding: const EdgeInsets.only(bottom: 8.0),
            child: Text(
              label ?? '',
              style: Theme.of(context).textTheme.caption,
            ),
          ),
        ),
        SizedBox(
          width: _kImageWidth,
          height: _kImageHeight,
          child: ClipRRect(
            borderRadius: const BorderRadius.all(Radius.circular(_kImageBorderRadius)),
            child: Image(image: image),
          ),
        ),
      ],
    );
  }
}
