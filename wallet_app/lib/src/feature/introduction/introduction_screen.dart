import 'dart:async';
import 'dart:math';

import 'package:collection/collection.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:lottie/lottie.dart';
import 'package:rxdart/rxdart.dart';

import '../../../environment.dart';
import '../../domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/animation_control_icon_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'widget/introduction_progress_stepper.dart';

// Nr of introduction pages to be shown
const _kNrOfPages = 3;

class IntroductionScreen extends StatefulWidget {
  const IntroductionScreen({super.key});

  @override
  State<IntroductionScreen> createState() => _IntroductionScreenState();
}

class _IntroductionScreenState extends State<IntroductionScreen> {
  final PageController _pageController = PageController();
  final Subject<String> _announcementStream = PublishSubject();
  StreamSubscription? _announcementStreamSubscription;

  final List<ScrollController> _scrollControllers = [
    ScrollController(debugLabel: 'intro_page_1'),
    ScrollController(debugLabel: 'intro_page_2'),
    ScrollController(debugLabel: 'intro_page_3'),
  ];

  /// Whether (Lottie) animations should be playing
  bool _playAnimations = true;

  /// The currently visible page
  double get _currentPage => _pageController.hasClients ? _pageController.page ?? 0 : 0;

  /// The currently visible page, without intermediate animation values
  int get _currentPageInt => (_currentPage + 0.5).toInt();

  /// The [ScrollController] associated to the current page, associated through [_currentPageInt].
  ScrollController? get _currentScrollController => _scrollControllers.elementAtOrNull(_currentPageInt);

  /// The scroll offset of the active page's [ScrollController]
  double get _currentScrollControllerPixelOffset {
    final scrollController = _currentScrollController;
    return (scrollController?.hasClients ?? false) ? scrollController!.position.pixels : 0;
  }

  bool get _showSkipSetupButton => kDebugMode && !Environment.isTest && Environment.mockRepositories;

  @override
  void initState() {
    super.initState();
    _announcementStreamSubscription = _announcementStream.debounceTime(const Duration(seconds: 3)).listen(_announce);
    _pageController.addListener(_onPageChanged);
    for (final scrollController in _scrollControllers) {
      scrollController.addListener(_onPageScrolled);
    }
  }

  @override
  void dispose() {
    _pageController.dispose();
    for (final scrollController in _scrollControllers) {
      scrollController.dispose();
    }
    _announcementStream.close();
    _announcementStreamSubscription?.cancel();
    super.dispose();
  }

  void _onPageChanged() => setState(() {});

  void _onPageScrolled() => setState(() {});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        Scaffold(
          restorationId: 'introduction_scaffold',
          appBar: WalletAppBar(
            leading: _buildBackButton(),
            automaticallyImplyLeading: false,
            actions: [
              const HelpIconButton(),
              if (_showSkipSetupButton) _buildSkipSetupButton(),
            ],
            title: FadeInAtOffset(
              scrollController: _currentScrollController,
              appearOffset: 38,
              visibleOffset: 58,
              child: TitleText(_resolveTitle()),
            ),
          ),
          body: PopScope(
            canPop: _currentPage == 0,
            onPopInvokedWithResult: (didPop, result) => didPop ? null : _onPreviousPagePressed(context),
            child: _buildContent(context),
          ),
        ),
        _buildGovernmentLabel(context),
      ],
    );
  }

  Widget _buildGovernmentLabel(BuildContext context) {
    final labelOffset = -2 * _currentScrollControllerPixelOffset;
    final normalizedOffset = min(labelOffset, 0).toDouble();
    return Positioned(
      top: normalizedOffset,
      left: 0,
      right: 0,
      child: Center(
        child: Semantics(
          attributedLabel: context.l10n.introductionWCAGDutchGovernmentLogoLabel.toAttributedString(context),
          child: Image.asset(
            WalletAssets.logo_rijksoverheid_label,
            height: context.isLandscape ? 64 : 88,
            fit: BoxFit.contain,
          ),
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: PageView(
            physics: const ClampingScrollPhysics(parent: RangeMaintainingScrollPhysics()),
            controller: _pageController,
            children: [
              _buildPage1(context),
              _buildPage2(context),
              _buildPage3(context),
            ],
          ),
        ),
        _buildControls(context),
      ],
    );
  }

  Widget _buildPage({
    required Key key,
    required BuildContext context,
    required String title,
    required String description,
    required String lottieAsset,
    ScrollController? controller,
  }) {
    return SafeArea(
      key: key,
      top: false,
      bottom: false,
      child: WalletScrollbar(
        controller: controller,
        child: CustomScrollView(
          controller: controller,
          physics: const AlwaysScrollableScrollPhysics(),
          slivers: [
            const SliverSizedBox(height: 24),
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    TitleText(title),
                    const SizedBox(height: 8),
                    BodyText(description),
                  ],
                ),
              ),
            ),
            const SliverSizedBox(height: 32),
            SliverFillRemaining(
              hasScrollBody: false,
              child: Stack(
                children: [
                  Container(
                    alignment: Alignment.center,
                    color: context.colorScheme.primaryContainer,
                    padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
                    child: Lottie.asset(
                      lottieAsset,
                      fit: BoxFit.contain,
                      animate: _playAnimations && !Environment.isTest,
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.all(16),
                    child: _buildPlayPauseButton(context),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPlayPauseButton(BuildContext context) {
    return AnimationControlIconButton(
      animationState: _playAnimations ? AnimationState.playing : AnimationState.paused,
      onPressed: () => setState(() => _playAnimations = !_playAnimations),
    );
  }

  Widget _buildPage1(BuildContext context) {
    return _buildPage(
      key: const Key('introductionPage1'),
      context: context,
      title: context.l10n.introductionPage1Title,
      description: context.l10n.introductionPage1Description,
      lottieAsset: WalletAssets.lottie_intro_1,
      controller: _scrollControllers[0],
    );
  }

  Widget _buildPage2(BuildContext context) {
    return _buildPage(
      key: const Key('introductionPage2'),
      context: context,
      title: context.l10n.introductionPage2Title,
      description: context.l10n.introductionPage2Description,
      lottieAsset: WalletAssets.lottie_intro_2,
      controller: _scrollControllers[1],
    );
  }

  Widget _buildPage3(BuildContext context) {
    return _buildPage(
      key: const Key('introductionPage3'),
      context: context,
      title: context.l10n.introductionPage3Title,
      description: context.l10n.introductionPage3Description,
      lottieAsset: WalletAssets.lottie_intro_3,
      controller: _scrollControllers[2],
    );
  }

  Widget _buildSkipSetupButton() {
    return IconButton(
      onPressed: () async {
        final navigator = Navigator.of(context);
        await context.read<SetupMockedWalletUseCase>().invoke();
        await navigator.pushReplacementNamed(WalletRoutes.dashboardRoute);
      },
      icon: const Icon(
        Icons.skip_next_outlined,
        semanticLabel: 'Skip Setup',
      ),
    );
  }

  Widget _buildControls(BuildContext context) {
    return Column(
      children: [
        SizedBox(height: context.isLandscape ? 8 : 16),
        IntroductionProgressStepper(
          currentStep: _currentPage,
          totalSteps: _kNrOfPages,
        ),
        ConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text.rich(context.l10n.introductionNextPageCta.toTextSpan(context)),
            onPressed: () => _onNextPressed(context),
            icon: const Icon(Icons.arrow_forward),
            key: Key('introductionNextPageCta_$_currentPageInt'),
          ),
          secondaryButton: TertiaryButton(
            text: Text.rich(context.l10n.introductionSkipCta.toTextSpan(context)),
            onPressed: () => _onSkipPressed(context),
            icon: const Icon(Icons.arrow_forward),
            key: const Key('introductionSkipCta'),
          ),
          hideSecondaryButton: _currentPage >= _kNrOfPages - 1.5,
        ),
      ],
    );
  }

  void _onNextPressed(BuildContext context) {
    final isOnLastPage = (_currentPage + 0.5).toInt() == _kNrOfPages - 1;
    if (isOnLastPage) {
      Navigator.of(context).restorablePushNamed(WalletRoutes.introductionPrivacyRoute);
    } else {
      _announcePage(_currentPage.toInt() + 2 /* +1 to account for zero based index, +1 for next page */);
      _pageController.nextPage(duration: kDefaultAnimationDuration, curve: Curves.easeOutCubic);
    }
  }

  void _onSkipPressed(BuildContext context) =>
      Navigator.of(context).restorablePushNamed(WalletRoutes.introductionPrivacyRoute);

  void _onPreviousPagePressed(BuildContext context) {
    _announcePage(_currentPage.toInt() + 0 /* +1 to account for zero based index, -1 for previous page */);
    _pageController.previousPage(duration: kDefaultAnimationDuration, curve: Curves.easeOutCubic);
  }

  Future<void> _announcePage(int page) async {
    final announcement = context.l10n.generalPageChangeWCAGAnnouncement(page, _kNrOfPages);
    _announcementStream.add(announcement);
  }

  void _announce(String announcement) => SemanticsService.announce(announcement, TextDirection.ltr);

  Widget? _buildBackButton() {
    if (_currentPage < 0.5) return null;
    return Opacity(
      opacity: _currentPage.clamp(0.0, 1.0),
      child: BackIconButton(
        onPressed: () => _onPreviousPagePressed(context),
      ),
    );
  }

  String _resolveTitle() {
    switch (_currentPageInt) {
      case 0:
        return context.l10n.introductionPage1Title;
      case 1:
        return context.l10n.introductionPage2Title;
      case 2:
        return context.l10n.introductionPage3Title;
    }
    throw UnsupportedError('Unknown page: $_currentPageInt');
  }
}
