import 'dart:math';

import 'package:after_layout/after_layout.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../environment.dart';
import '../../domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/text_icon_button.dart';
import 'page/introduction_page.dart';
import 'page/introduction_privacy_page.dart';
import 'widget/introduction_progress_stepper.dart';

// Progress constants
const _kNrOfPages = 5;

// Semantic constants
const _kBackButtonSortKey = -1.0;

class IntroductionScreen extends StatefulWidget {
  const IntroductionScreen({Key? key}) : super(key: key);

  @override
  State<IntroductionScreen> createState() => _IntroductionScreenState();
}

class _IntroductionScreenState extends State<IntroductionScreen> with AfterLayoutMixin<IntroductionScreen> {
  final PageController _pageController = PageController();
  final GlobalKey _placeholderKey = GlobalKey();
  OverlayEntry? _overlayEntry;

  double get _currentPage => _pageController.hasClients ? _pageController.page ?? 0 : 0;

  @override
  void initState() {
    super.initState();
    _pageController.addListener(_onPageChanged);
  }

  void _onPageChanged() => setState(() => _overlayEntry?.markNeedsBuild());

  @override
  void dispose() {
    _pageController.dispose();
    _overlayEntry?.remove();
    _overlayEntry?.dispose();
    super.dispose();
  }

  @override
  void afterFirstLayout(BuildContext context) async {
    _overlayEntry = _createOverlayEntry();
    if (_overlayEntry != null) {
      Overlay.of(context).insert(_overlayEntry!);
    }
  }

  OverlayEntry? _createOverlayEntry() {
    double? yCache;
    // Local helper method to build the positioned stepper, to keep things consise.
    Widget buildStepper(double y) {
      return Positioned(
        left: 0,
        top: y,
        child: _buildProgressStepper(_currentPage),
      );
    }

    return OverlayEntry(
      builder: (context) {
        // Hide stepper in landscape
        if (context.isLandscape) return const SizedBox.shrink();
        // If we already know where to position the stepper, simply build it!
        if (yCache != null) return buildStepper(yCache!);
        // Hide stepper when the screen or placeholder isn't mounted (e.g. when navigating away)
        if (!mounted || _placeholderKey.currentContext?.mounted == false) return const SizedBox.shrink();
        // Hide stepper when we can't determine the position of the placeholder
        RenderObject? renderBox = _placeholderKey.currentContext?.findRenderObject();
        if (renderBox == null || renderBox is! RenderBox) return const SizedBox.shrink();
        // Set the yCache for future rebuilds
        yCache = renderBox.localToGlobal(Offset.zero).dy;
        // Render the actual progress stepper!
        return Positioned(
          left: 0,
          top: yCache,
          child: _buildProgressStepper(_currentPage),
        );
      },
    );
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
            _buildAppIntroPage1(context),
            _buildAppIntroPage2(context),
            _buildAppIntroPage3(context),
            _buildAppIntroPage4(context),
            _buildAppPrivacyPage(context),
          ],
        ),
        Semantics(
          sortKey: const OrdinalSortKey(_kBackButtonSortKey),
          explicitChildNodes: true,
          child: _buildBackButton(),
        ),
      ],
    );
  }

  Widget _buildAppIntroPage1(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_intro_page_1.png'),
      title: context.l10n.introductionPage1Title,
      subtitle: context.l10n.introductionPage1Description,
      header: _buildProgressStepperPlaceHolder(step: 1, key: _placeholderKey),
      footer: _buildBottomSection(context),
    );
  }

  Widget _buildAppIntroPage2(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_intro_page_2.png'),
      title: context.l10n.introductionPage2Title,
      subtitle: context.l10n.introductionPage2Description,
      bulletPoints: context.l10n.introductionPage2BulletPoints.split('\n'),
      header: _buildProgressStepperPlaceHolder(step: 2),
      footer: _buildBottomSection(context),
    );
  }

  Widget _buildAppIntroPage3(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_intro_page_3.png'),
      title: context.l10n.introductionPage3Title,
      subtitle: context.l10n.introductionPage3Description,
      header: _buildProgressStepperPlaceHolder(step: 3),
      footer: _buildBottomSection(context),
    );
  }

  Widget _buildAppIntroPage4(BuildContext context) {
    return IntroductionPage(
      image: const AssetImage('assets/non-free/images/image_intro_page_4.png'),
      title: context.l10n.introductionPage4Title,
      bulletPoints: context.l10n.introductionPage4BulletPoints.split('\n'),
      header: _buildProgressStepperPlaceHolder(step: 4),
      footer: _buildBottomSection(context),
    );
  }

  Widget _buildAppPrivacyPage(BuildContext context) {
    return IntroductionPrivacyPage(
      footer: _buildPrivacyBottomSection(context),
    );
  }

  Widget _buildProgressStepper(double currentStep) {
    const indexOfLastPageWithStepper = (_kNrOfPages - 2);
    final alpha = min(1 - (_currentPage - indexOfLastPageWithStepper), 1.0);
    return Opacity(
      opacity: context.isLandscape ? 0 : alpha,
      child: ExcludeSemantics(
        child: Padding(
          padding: const EdgeInsets.only(left: 16, right: 16, top: 16),
          child: IntroductionProgressStepper(currentStep: currentStep, totalSteps: _kNrOfPages - 1),
        ),
      ),
    );
  }

  Widget _buildProgressStepperPlaceHolder({required int step, Key? key}) {
    const stepperPadding = 8.0;
    const stepperWidth = (_kNrOfPages - 1) * 16 + 8.0;
    // This [Container] construction is only there to make sure the accessibility rectangle is drawn correctly.
    return Container(
      key: key,
      margin: const EdgeInsets.only(left: stepperPadding),
      alignment: Alignment.centerLeft,
      child: Transform.translate(
        offset: const Offset(0, stepperPadding),
        child: Semantics(
          container: true,
          label: context.l10n.introductionWCAGCurrentPageAnnouncement(step, _kNrOfPages - 1),
          child: const SizedBox(
            height: 22,
            width: stepperWidth,
          ),
        ),
      ),
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
    final isOnLastPage = (_currentPage + 0.5).toInt() == _kNrOfPages - 1;
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
            child: const Text('Skip Setup'),
          ),
        ],
      );
    }
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildNextButton(),
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
          _buildNextButton(),
        ],
      ),
    );
  }

  Widget _buildNextButton() {
    return ElevatedButton(
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
