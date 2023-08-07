import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/icon_row.dart';

const _kCoverHeaderImageDesiredHeight = 250.0;
const _kCoverHeaderLabelImage = 'assets/non-free/images/logo_rijksoverheid_label.png';

class IntroductionPage extends StatelessWidget {
  final ImageProvider image;
  final Widget? header, footer;
  final String title;
  final String? subtitle;
  final List<String> bulletPoints;
  final ScrollController? scrollController;

  const IntroductionPage({
    required this.image,
    this.header,
    this.footer,
    required this.title,
    this.subtitle,
    this.bulletPoints = const [],
    this.scrollController,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return PrimaryScrollController(
      controller: scrollController ?? ScrollController(),
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
        _buildPortraitImage(context),
        Expanded(
          child: Scrollbar(
            child: ListView(
              padding: const EdgeInsets.only(top: 16),
              children: [
                header != null ? header! : const SizedBox.shrink(),
                const SizedBox(height: 8),
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
        Expanded(child: _buildLandscapeImage(context)),
        Expanded(
          child: SafeArea(
            bottom: false,
            child: Scrollbar(
              child: Column(
                children: [
                  Expanded(
                    child: SingleChildScrollView(
                      padding: const EdgeInsets.only(top: 8),
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

  Widget _buildLandscapeImage(BuildContext context) {
    return Stack(
      fit: StackFit.passthrough,
      children: [
        Positioned.fill(
          child: Image(
            image: image,
            fit: BoxFit.cover,
          ),
        ),
        _buildLogo(context),
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
        _buildLogo(context),
      ],
    );
  }

  Widget _buildLogo(BuildContext context) {
    return Align(
      alignment: Alignment.topCenter,
      child: Semantics(
        label: context.l10n.introductionWCAGDutchGovernmentLogoLabel,
        child: Image.asset(
          _kCoverHeaderLabelImage,
          fit: BoxFit.cover,
        ),
      ),
    );
  }

  Widget _buildInfoSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 16),
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
          SizedBox(height: subtitle == null ? 0 : 8),
          if (subtitle != null)
            Text(
              subtitle!,
              style: context.textTheme.bodyLarge,
              textAlign: TextAlign.start,
            ),
          _buildBulletPoints(context),
        ],
      ),
    );
  }

  Widget _buildBulletPoints(BuildContext context) {
    if (bulletPoints.isEmpty) return const SizedBox.shrink();
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: bulletPoints.map((point) {
          return IconRow(
            padding: const EdgeInsets.symmetric(vertical: 4),
            icon: SizedBox(
              height: 24,
              width: 24,
              child: Icon(
                Icons.check,
                color: context.colorScheme.primary,
                size: 18,
              ),
            ),
            text: Text(point),
          );
        }).toList(),
      ),
    );
  }
}
