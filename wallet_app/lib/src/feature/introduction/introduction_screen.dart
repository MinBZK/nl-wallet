import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../environment.dart';
import '../../domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/rounded_back_button.dart';
import '../common/widget/button/text_icon_button.dart';
import 'page/introduction_page.dart';
import 'widget/introduction_progress_stepper.dart';

// Nr of introduction pages to be shown
const _kNrOfPages = 4;

// Strength of the parallax effect, used to translate the page indicator
const _kParallaxStrength = 0.4;

// Semantic constants
const _kBackButtonSortKey = -1.0;

class IntroductionScreen extends StatefulWidget {
  const IntroductionScreen({Key? key}) : super(key: key);

  @override
  State<IntroductionScreen> createState() => _IntroductionScreenState();
}

class _IntroductionScreenState extends State<IntroductionScreen> {
  final PageController _pageController = PageController();

  /// This key is assigned to a widget who's location will be used to place the page indicator
  final GlobalKey _pageIndicatorPositionPlaceholderKey = GlobalKey();

  final List<ScrollController> _scrollControllers = [
    ScrollController(debugLabel: 'intro_page_1'),
    ScrollController(debugLabel: 'intro_page_2'),
    ScrollController(debugLabel: 'intro_page_3'),
    ScrollController(debugLabel: 'intro_page_4')
  ];

  /// The currently visible page
  double get _currentPage => _pageController.hasClients ? _pageController.page ?? 0 : 0;

  /// The currently visible page, without intermediate animation values
  int get _currentPageInt => (_currentPage + 0.5).toInt();

  /// The [ScrollController] associated to the current page, associated through [_currentPageInt].
  ScrollController? get _currentScrollController => _scrollControllers.elementAtOrNull(_currentPageInt);

  /// The scroll offset of the active page's [ScrollController]
  double get _currentScrollControllerPixelOffset {
    final scrollController = _currentScrollController;
    return (scrollController?.hasClients == true) ? scrollController!.position.pixels : 0;
  }

  /// Internally used cache position, rely on [_pageIndicatorYPosition] instead.
  double? _pageIndicatorYPosCache;

  /// The base y position of the page indicator, determined by finding the position of the
  /// widget referenced by the [_pageIndicatorPositionPlaceholderKey], as the page indicator should sit right
  /// above it.
  double? get _pageIndicatorYPosition {
    /// If we already know where to position the stepper, simply return it!
    if (_pageIndicatorYPosCache != null) return _pageIndicatorYPosCache;

    /// Hide indicator when the screen or placeholder isn't mounted
    if (!mounted || _pageIndicatorPositionPlaceholderKey.currentContext?.mounted == false) return null;

    /// Hide indicator when we can't determine the position of the placeholder
    RenderObject? renderBox = _pageIndicatorPositionPlaceholderKey.currentContext?.findRenderObject();
    if (renderBox == null || renderBox is! RenderBox) return null;

    /// Make sure we consider any scrollOffset that happened before caching
    final scrollOffset = _currentScrollControllerPixelOffset;

    /// Set the yCache for future rebuilds
    _pageIndicatorYPosCache = renderBox.localToGlobal(Offset.zero).dy + scrollOffset;
    return _pageIndicatorYPosCache;
  }

  @override
  void initState() {
    super.initState();
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
    super.dispose();
  }

  void _onPageChanged() => setState(() {});

  void _onPageScrolled() => setState(() {});

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
          physics: const ClampingScrollPhysics(parent: RangeMaintainingScrollPhysics()),
          controller: _pageController,
          children: [
            _buildAppIntroPage1(context),
            _buildAppIntroPage2(context),
            _buildAppIntroPage3(context),
            _buildAppIntroPage4(context),
          ],
        ),
        _buildPositionedPageIndicator(),
        Semantics(
          sortKey: const OrdinalSortKey(_kBackButtonSortKey),
          explicitChildNodes: true,
          child: _buildBackButton(),
        ),
      ],
    );
  }

  Widget _buildAppIntroPage1(BuildContext context) {
    const pageIndex = 0;
    return IntroductionPage(
      image: const AssetImage(WalletAssets.image_intro_page_1),
      title: context.l10n.introductionPage1Title,
      subtitle: context.l10n.introductionPage1Description,
      header: _buildPageIndicatorPlaceholder(
        step: pageIndex + 1,
        key: _currentPageInt == pageIndex ? _pageIndicatorPositionPlaceholderKey : null,
      ),
      footer: _buildBottomSection(context),
      scrollController: _scrollControllers[pageIndex],
    );
  }

  Widget _buildAppIntroPage2(BuildContext context) {
    const pageIndex = 1;
    return IntroductionPage(
      image: const AssetImage(WalletAssets.image_intro_page_2),
      title: context.l10n.introductionPage2Title,
      subtitle: context.l10n.introductionPage2Description,
      bulletPoints: context.l10n.introductionPage2BulletPoints.split('\n'),
      header: _buildPageIndicatorPlaceholder(
        step: pageIndex + 1,
        key: _currentPageInt == pageIndex ? _pageIndicatorPositionPlaceholderKey : null,
      ),
      footer: _buildBottomSection(context),
      scrollController: _scrollControllers[pageIndex],
    );
  }

  Widget _buildAppIntroPage3(BuildContext context) {
    const pageIndex = 2;
    return IntroductionPage(
      image: const AssetImage(WalletAssets.image_intro_page_3),
      title: context.l10n.introductionPage3Title,
      subtitle: context.l10n.introductionPage3Description,
      header: _buildPageIndicatorPlaceholder(
        step: pageIndex + 1,
        key: _currentPageInt == pageIndex ? _pageIndicatorPositionPlaceholderKey : null,
      ),
      footer: _buildBottomSection(context),
      scrollController: _scrollControllers[pageIndex],
    );
  }

  Widget _buildAppIntroPage4(BuildContext context) {
    const pageIndex = 3;
    return IntroductionPage(
      image: const AssetImage(WalletAssets.image_intro_page_4),
      title: context.l10n.introductionPage4Title,
      bulletPoints: context.l10n.introductionPage4BulletPoints.split('\n'),
      header: _buildPageIndicatorPlaceholder(
        step: pageIndex + 1,
        key: _currentPageInt == pageIndex ? _pageIndicatorPositionPlaceholderKey : null,
      ),
      footer: _buildBottomSection(context),
      scrollController: _scrollControllers[pageIndex],
    );
  }

  Widget _buildPositionedPageIndicator() {
    /// Hide the page indicator in landscape layout
    if (context.isLandscape) return const SizedBox.shrink();

    /// Figure out where to place the pageIndicator (y position)
    var yPos = _pageIndicatorYPosition;
    if (yPos == null) {
      /// y could not be resolved yet, this happens on the first build, when the position determining
      /// widget has not been laid out yet. Trigger a rebuild to get its position on the next frame.
      WidgetsBinding.instance.addPostFrameCallback((timeStamp) => setState(() {}));
      return const SizedBox.shrink();
    }
    yPos -= (_currentScrollControllerPixelOffset * _kParallaxStrength);

    /// Next to translating it, also fade it out when it moves up
    const offsetUntilFade = 40.0;
    final normalized = _currentScrollControllerPixelOffset / offsetUntilFade;
    double scrollBasedOpacity = (1 - normalized).clamp(0, 1);

    /// Finally position the indicator with the calculated position [yPos] and opacity [scrollBasedOpacity]
    return Positioned(
      left: 0,
      top: yPos,
      child: Opacity(
        opacity: scrollBasedOpacity,
        child: _buildPageIndicator(_currentPage),
      ),
    );
  }

  Widget _buildPageIndicator(double currentStep) {
    return ExcludeSemantics(
      child: Padding(
        padding: const EdgeInsets.only(left: 16, right: 16, top: 16),
        child: IntroductionProgressStepper(
          currentStep: currentStep,
          totalSteps: _kNrOfPages,
        ),
      ),
    );
  }

  /// Builds a widget that:
  /// - Makes sure the correct space is reserved to actually draw the rendered stepper (which is done in an Overlay)
  /// - Announces the current page number when a screen reader is enabled
  /// - Positions the [SizedBox] used to draw the accessibility indicator
  Widget _buildPageIndicatorPlaceholder({required int step, Key? key}) {
    const indicatorPadding = 8.0;
    const indicatorWidth = _kNrOfPages * 16 + 8.0;
    // This [Container] construction is only there to make sure the accessibility rectangle is drawn correctly.
    return Container(
      key: key,
      margin: const EdgeInsets.only(left: indicatorPadding),
      alignment: Alignment.centerLeft,
      child: Transform.translate(
        offset: const Offset(0, indicatorPadding),
        child: Semantics(
          container: true,
          label: context.l10n.introductionWCAGCurrentPageAnnouncement(step, _kNrOfPages - 1),
          child: const SizedBox(
            height: 22,
            width: indicatorWidth,
          ),
        ),
      ),
    );
  }

  void _onNextPressed(BuildContext context) {
    final isOnLastPage = (_currentPage + 0.5).toInt() == _kNrOfPages - 1;
    if (isOnLastPage) {
      Navigator.restorablePushNamed(context, WalletRoutes.introductionExpectationsRoute);
    } else {
      _pageController.nextPage(duration: kDefaultAnimationDuration, curve: Curves.easeOutCubic);
    }
  }

  void _onSkipPressed(BuildContext context) =>
      Navigator.restorablePushNamed(context, WalletRoutes.introductionExpectationsRoute);

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
    return Opacity(
      opacity: (_currentPage).clamp(0.0, 1.0),
      child: const SafeArea(
        child: RoundedBackButton(),
      ),
    );
  }
}
