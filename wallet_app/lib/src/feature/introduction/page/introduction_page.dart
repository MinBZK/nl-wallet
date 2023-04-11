import 'dart:math';

import 'package:flutter/material.dart';

const _kCoverHeaderImageDesiredHeight = 400.0;
const _kCoverHeaderImageMaxFraction = 0.5;
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
    return OrientationBuilder(builder: (context, orientation) {
      if (orientation == Orientation.portrait) {
        return _buildPortrait(context);
      } else {
        return _buildLandscape(context);
      }
    });
  }

  Widget _buildPortrait(BuildContext context) {
    return Column(
      children: [
        _buildPortraitImage(context),
        _buildTextHeadline(context),
      ],
    );
  }

  Widget _buildLandscape(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: _buildLandscapeImage(),
        ),
        Expanded(
          child: SafeArea(
            child: Align(
              alignment: Alignment.topCenter,
              child: _buildTextHeadline(context),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildLandscapeImage() {
    return Stack(
      fit: StackFit.passthrough,
      children: [
        Positioned.fill(
          child: Image(
            image: image,
            fit: BoxFit.cover,
          ),
        ),
        Align(
          alignment: Alignment.topCenter,
          child: Image.asset(
            _kCoverHeaderLabelImage,
            fit: BoxFit.cover,
          ),
        ),
      ],
    );
  }

  Widget _buildPortraitImage(BuildContext context) {
    final textScaleFactor = MediaQuery.of(context).textScaleFactor;
    final screenHeight = MediaQuery.of(context).size.height;
    final maxFractionHeight = screenHeight * _kCoverHeaderImageMaxFraction;
    return Stack(
      children: [
        SizedBox(
          width: double.infinity,
          height: min(_kCoverHeaderImageDesiredHeight, maxFractionHeight) / textScaleFactor,
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
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Text(
        title,
        style: Theme.of(context).textTheme.displayLarge,
        textAlign: TextAlign.center,
        textScaleFactor: 1,
      ),
    );
  }
}
