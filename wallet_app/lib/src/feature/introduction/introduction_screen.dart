import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../environment.dart';
import '../../domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/text_icon_button.dart';
import '../common/widget/placeholder_screen.dart';
import 'page/introduction_page.dart';
import 'page/introduction_privacy_page.dart';
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
            _buildAppIntroductionPage(context),
            _buildAppBenefitsPage(context),
            _buildAppSecurityPage(context),
            _buildAppPrivacyPage(context),
          ],
        ),
        Semantics(
          sortKey: const OrdinalSortKey(-1),
          explicitChildNodes: true,
          child: _buildBackButton(),
        ),
      ],
    );
  }

  Widget _buildAppIntroductionPage(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_introduction_app_introduction.png'),
      title: context.l10n.introductionAppIntroPageTitle,
      subtitle: context.l10n.introductionAppIntroPageSubtitle,
      header: _buildProgressStepper(_currentPage),
      footer: _buildBottomSection(context),
    );
  }

  Widget _buildAppBenefitsPage(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_introduction_app_benefits.png'),
      title: context.l10n.introductionAppBenefitsPageTitle,
      subtitle: context.l10n.introductionAppBenefitsPageSubtitle,
      header: _buildProgressStepper(_currentPage),
      footer: _buildBottomSection(context),
    );
  }

  Widget _buildAppSecurityPage(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_introduction_app_security.png'),
      title: context.l10n.introductionAppSecurityPageTitle,
      subtitle: context.l10n.introductionAppSecurityPageSubtitle,
      header: _buildProgressStepper(_currentPage),
      footer: _buildBottomSection(context),
    );
  }

  Widget _buildAppPrivacyPage(BuildContext context) {
    return IntroductionPrivacyPage(footer: _buildPrivacyBottomSection(context));
  }

  Widget _buildProgressStepper(double currentStep) {
    return Padding(
      padding: const EdgeInsets.only(top: 32, left: 16, right: 16),
      child: IntroductionProgressStepper(currentStep: currentStep, totalSteps: _kNrOfPages - 1),
    );
  }

  Widget _buildPrivacyPolicyCta(BuildContext context) {
    return TextIconButton(
      key: const Key('introductionPrivacyPolicyCta'),
      icon: Icons.arrow_forward,
      onPressed: () => PlaceholderScreen.show(context, secured: false),
      child: Text(context.l10n.introductionPrivacyPolicyCta),
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

  void _onSkipPressed(BuildContext context) => _pageController.animateToPage(
        _kNrOfPages - 1,
        duration: kDefaultAnimationDuration,
        curve: Curves.easeOutCubic,
      );

  void _onBackPressed(BuildContext context) {
    _pageController.previousPage(duration: kDefaultAnimationDuration, curve: Curves.easeOutCubic);
  }

  Widget _buildBottomSection(BuildContext context) {
    Widget skipButton = TextIconButton(
      key: const Key('introductionSkipCta'),
      iconPosition: IconPosition.start,
      centerChild: false,
      onPressed: () => _onSkipPressed(context),
      child: Text(context.l10n.introductionSkipCta),
    );

    //FIXME: This kDebugMode & isTest check is to be replaced a more elaborate deeplink
    //FIXME: setup that allows us to configure the app with (custom) mock data.
    if (kDebugMode && !Environment.isTest) {
      // Inject the skip setup button
      skipButton = Row(
        mainAxisSize: MainAxisSize.max,
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          skipButton,
          TextIconButton(
            iconPosition: IconPosition.start,
            centerChild: false,
            onPressed: () async {
              final navigator = Navigator.of(context);
              await context.read<SetupMockedWalletUseCase>().invoke();
              navigator.pushReplacementNamed(WalletRoutes.homeRoute);
            },
            child: const Text('Skip Setup (Dev)'),
          ),
        ],
      );
    }
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          ElevatedButton(
            key: const Key('introductionNextPageCta'),
            onPressed: () => _onNextPressed(context),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.arrow_forward, size: 16),
                const SizedBox(width: 8),
                Text(
                  context.l10n.introductionNextPageCta,
                  key: const Key('introductionNextPageCtaText'),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
          skipButton,
        ],
      ),
    );
  }

  Widget _buildPrivacyBottomSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildPrivacyPolicyCta(context),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => _onNextPressed(context),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.arrow_forward, size: 16),
                const SizedBox(width: 8),
                Text(
                  context.l10n.introductionNextPageCta,
                  key: const Key('introductionNextPageCtaText'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBackButton() {
    /// Slightly awkward Widget Setup to make sure tap target is 48px (accessibility requirement)
    final backButton = SizedBox(
      width: 48,
      height: 48,
      child: Material(
        color: Colors.transparent,
        clipBehavior: Clip.antiAlias,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(24)),
        child: Semantics(
          key: const Key('introductionBackCta'),
          excludeSemantics: _currentPage < 1.0,
          button: true,
          tooltip: context.l10n.generalWCAGBack,
          child: InkWell(
            onTap: () => _onBackPressed(context),
            child: Container(
              margin: const EdgeInsets.all(8),
              alignment: Alignment.center,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: context.colorScheme.background,
              ),
              child: Icon(
                Icons.arrow_back,
                color: context.colorScheme.onBackground,
              ),
            ),
          ),
        ),
      ),
    );
    return Opacity(
      opacity: (_currentPage).clamp(0.0, 1.0),
      child: SafeArea(child: backButton),
    );
  }
}
