import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

const _kCoverHeaderImageHeight = 400.0;
const _kCoverHeaderLabelImage = 'assets/non-free/images/logo_rijksoverheid_label.png';

class IntroductionPage extends StatelessWidget {
  final ImageProvider image;
  final String title;
  final Widget progressStepper;
  final Widget? secondaryCta;
  final VoidCallback onNextPressed;
  final VoidCallback? onBackPressed;

  const IntroductionPage({
    required this.image,
    required this.title,
    required this.progressStepper,
    required this.onNextPressed,
    this.secondaryCta,
    this.onBackPressed,
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
            progressStepper,
            const Spacer(),
            if (secondaryCta != null) secondaryCta!,
            _buildNextButton(context),
          ],
        ),
        if (onBackPressed != null) _buildBackButton(),
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

  Widget _buildNextButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 24, 16, 32),
      child: ElevatedButton(
        onPressed: onNextPressed,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.arrow_forward, size: 16),
            const SizedBox(width: 8),
            Text(AppLocalizations.of(context).introductionNextPageCta),
          ],
        ),
      ),
    );
  }

  Widget _buildBackButton() {
    return SafeArea(
      minimum: const EdgeInsets.only(left: 8),
      child: Material(
        color: Colors.white,
        borderRadius: BorderRadius.circular(16),
        clipBehavior: Clip.hardEdge,
        child: InkWell(
          onTap: onBackPressed,
          child: Ink(
            width: 32,
            height: 32,
            child: const Icon(Icons.arrow_back),
          ),
        ),
      ),
    );
  }
}
