import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import '../../util/extension/num_extensions.dart';
import '../../wallet_constants.dart';
import '../../wallet_routes.dart';
import '../common/widget/placeholder_screen.dart';
import '../common/widget/text_icon_button.dart';
import 'page/introduction_page.dart';
import 'widget/introduction_progress_stepper.dart';

const int _kNrOfPages = 4;

class IntroductionScreen extends StatefulWidget {
  const IntroductionScreen({Key? key}) : super(key: key);

  @override
  State<IntroductionScreen> createState() => _IntroductionScreenState();
}

class _IntroductionScreenState extends State<IntroductionScreen> {
  final PageController _pageController = PageController();

  double get _currentPage => _pageController.hasClients ? _pageController.page ?? 0 : 0;

  @override
  void initState() {
    super.initState();
    _pageController.addListener(_onPageChanged);
  }

  void _onPageChanged() {
    setState(() {});
  }

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'introduction_scaffold',
      body: WillPopScope(
        onWillPop: () async {
          final canGoBack = _currentPage >= 1;
          if (canGoBack) _onBackPressed(context);
          return !canGoBack;
        },
        child: _buildPager(context),
      ),
    );
  }

  Widget _buildPager(BuildContext context) {
    return Stack(
      children: [
        PageView(
          controller: _pageController,
          children: [
            _buildAppDisclaimerPage(context),
            _buildAppIntroductionPage(context),
            _buildAppBenefitsPage(context),
            _buildAppSecurityPage(context),
          ],
        ),
        _buildBackButton(),
        Align(
          alignment: Alignment.bottomCenter,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildProgressStepper(_currentPage),
              const SizedBox(height: 24),
              _buildSecondaryCta(context),
              _buildNextButton(context),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildAppDisclaimerPage(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_introduction_app_disclaimer.png'),
      title: AppLocalizations.of(context).introductionAppDisclaimerPageTitle,
    );
  }

  Widget _buildAppIntroductionPage(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_introduction_app_introduction.png'),
      title: AppLocalizations.of(context).introductionAppIntroPageTitle,
    );
  }

  Widget _buildAppBenefitsPage(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_introduction_app_benefits.png'),
      title: AppLocalizations.of(context).introductionAppBenefitsPageTitle,
    );
  }

  Widget _buildAppSecurityPage(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_introduction_app_security.png'),
      title: AppLocalizations.of(context).introductionAppSecurityPageTitle,
    );
  }

  Widget _buildProgressStepper(double currentStep) {
    return IntroductionProgressStepper(currentStep: currentStep, totalSteps: _kNrOfPages);
  }

  Widget _buildSecondaryCta(BuildContext context) {
    final mainVisiblePage = (_currentPage + 0.5).floor();
    if (mainVisiblePage == 0 /* _buildAppDisclaimerPage */) {
      final opacity = 1.0 - _currentPage.clamp(0, 0.5).normalize(0, 0.5);
      return _buildLanguageButton(opacity);
    } else if (mainVisiblePage == 3 /* _buildAppSecurityPage */) {
      final opacity = _currentPage.clamp(2.5, 3).normalize(2.5, 3);
      return _buildPrivacyPolicyCta(context, opacity.toDouble());
    }
    // Empty button, to make sure content above the 'secondary' button doesn't move around.
    return const TextButton(onPressed: null, child: Text(''));
  }

  Widget _buildLanguageButton(double opacity) {
    final Widget result;
    if (kDebugMode) {
      result = TextIconButton(
        icon: Icons.skip_next,
        iconPosition: IconPosition.start,
        onPressed: () async {
          final navigator = Navigator.of(context);
          await context.read<SetupMockedWalletUseCase>().invoke();
          navigator.pushReplacementNamed(WalletRoutes.homeRoute);
        },
        centerChild: false,
        child: const Text('SKIP INTRO (DEV ONLY)'),
      );
    } else {
      result = TextIconButton(
        icon: Icons.language,
        iconPosition: IconPosition.start,
        onPressed: () => PlaceholderScreen.show(
          context,
          AppLocalizations.of(context).introductionLanguageSelectCta,
          secured: false,
        ),
        centerChild: false,
        child: Text(AppLocalizations.of(context).introductionLanguageSelectCta),
      );
    }
    return Opacity(opacity: opacity, child: result);
  }

  Widget _buildPrivacyPolicyCta(BuildContext context, double opacity) {
    return Opacity(
      opacity: opacity,
      child: TextIconButton(
        icon: Icons.arrow_forward,
        onPressed: () => PlaceholderScreen.show(
          context,
          AppLocalizations.of(context).introductionPrivacyPolicyCta,
          secured: false,
        ),
        child: Text(AppLocalizations.of(context).introductionPrivacyPolicyCta),
      ),
    );
  }

  void _onNextPressed(BuildContext context) {
    final isOnLastPage = (_currentPage + 0.5).toInt() == (_kNrOfPages - 1);
    if (isOnLastPage) {
      Navigator.restorablePushReplacementNamed(context, WalletRoutes.setupSecurityRoute);
    } else {
      _pageController.nextPage(duration: kDefaultAnimationDuration, curve: Curves.easeOutCubic);
    }
  }

  void _onBackPressed(BuildContext context) {
    _pageController.previousPage(duration: kDefaultAnimationDuration, curve: Curves.easeOutCubic);
  }

  Widget _buildNextButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: ElevatedButton(
        onPressed: () => _onNextPressed(context),
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
    final backButton = SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Material(
          color: Colors.white,
          borderRadius: BorderRadius.circular(16),
          clipBehavior: Clip.hardEdge,
          child: InkWell(
            onTap: () => _onBackPressed(context),
            child: Ink(
              width: 32,
              height: 32,
              child: const Icon(Icons.arrow_back),
            ),
          ),
        ),
      ),
    );
    return Opacity(opacity: (_currentPage).clamp(0.0, 1.0), child: backButton);
  }
}
