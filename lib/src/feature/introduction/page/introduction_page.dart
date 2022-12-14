import 'package:flutter/material.dart';

const _kCoverHeaderImageHeight = 400.0;
const _kCoverHeaderLabelImage = 'assets/non-free/images/logo_rijksoverheid_label.png';

class IntroductionPage extends StatelessWidget {
  final ImageProvider image;
  final String title;

  const IntroductionPage({
    required this.image,
    required this.title,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        Column(
          children: [
            _buildHeaderImages(),
            _buildTextHeadline(context),
          ],
        ),
      ],
    );
  }

  Widget _buildHeaderImages() {
    return Stack(
      children: [
        SizedBox(
          width: double.infinity,
          height: _kCoverHeaderImageHeight,
          child: Image(image: image, fit: BoxFit.cover),
        ),
        Align(
          alignment: Alignment.topCenter,
          child: Image.asset(_kCoverHeaderLabelImage, fit: BoxFit.cover),
        ),
      ],
    );
  }

  Widget _buildTextHeadline(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 32, horizontal: 16),
      child: Text(
        title,
        style: Theme.of(context).textTheme.headline1,
        textAlign: TextAlign.center,
      ),
    );
  }
}
