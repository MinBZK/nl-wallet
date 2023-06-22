import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

const _kCoverHeaderImageDesiredHeight = 250.0;
const _kCoverHeaderLabelImage = 'assets/non-free/images/logo_rijksoverheid_label.png';

class IntroductionPage extends StatelessWidget {
  final ImageProvider image;
  final Widget? header, footer;
  final String title, subtitle;

  const IntroductionPage({
    required this.image,
    this.header,
    this.footer,
    required this.title,
    required this.subtitle,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return PrimaryScrollController(
      controller: ScrollController(),
      child: OrientationBuilder(builder: (context, orientation) {
        if (orientation == Orientation.portrait) {
          return _buildPortrait(context);
        } else {
          return _buildLandscape(context);
        }
      }),
    );
  }

  Widget _buildPortrait(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: Scrollbar(
            child: ListView(
              padding: EdgeInsets.zero,
              children: [
                _buildPortraitImage(context),
                header != null ? header! : const SizedBox.shrink(),
                _buildInfoSection(context),
              ],
            ),
          ),
        ),
        if (footer != null) footer!,
      ],
    );
  }

  Widget _buildLandscape(BuildContext context) {
    return Row(
      children: [
        Expanded(child: _buildLandscapeImage()),
        Expanded(
          child: SafeArea(
            child: Scrollbar(
              child: Column(
                children: [
                  Expanded(
                    child: SingleChildScrollView(
                      padding: EdgeInsets.zero,
                      child: _buildInfoSection(context),
                    ),
                  ),
                  if (footer != null) footer!,
                ],
              ),
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
    return Stack(
      children: [
        SizedBox(
          width: double.infinity,
          height: _kCoverHeaderImageDesiredHeight,
          child: Image(image: image, fit: BoxFit.cover),
        ),
        Align(
          alignment: Alignment.topCenter,
          child: Image.asset(_kCoverHeaderLabelImage, fit: BoxFit.cover),
        ),
      ],
    );
  }

  Widget _buildInfoSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: context.textTheme.displayLarge,
            textAlign: TextAlign.start,
            textScaleFactor: 1,
          ),
          const SizedBox(height: 8),
          Text(
            subtitle,
            style: context.textTheme.bodyLarge,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }
}
