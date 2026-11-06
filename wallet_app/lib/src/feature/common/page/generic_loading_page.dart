import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../environment.dart';
import '../../../data/service/announcement_service.dart';
import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';
import '../widget/button/list_button.dart';
import '../widget/loading_indicator.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_app_bar.dart';

const double _kContextLogoSpacing = 40;

const double _kHorizontalPadding = 40;

/// A reusable widget for displaying a loading state with optional cancellation.
///
/// This page includes a customizable title, description, app bar, and contextual image.
/// A central loading indicator is shown by default, and a cancel button is displayed
/// at the bottom if [onCancel] is provided. The widget manages accessibility focus
/// for the title to ensure it's announced when the widget appears on screen.
class GenericLoadingPage extends StatefulWidget {
  /// The main title shown at the top of the loading page, displayed in large text and centered.
  final String title;

  /// Optional supporting text displayed below the title, centered on the page.
  ///
  /// Leave it out when the [title] carries the whole message, so a screen reader reads it in one go.
  final String? description;

  /// Optional callback triggered when the cancel button is pressed.
  ///
  /// If null, no cancel button is shown.
  final VoidCallback? onCancel;

  /// Text displayed in the cancel button.
  ///
  /// Defaults to [l10n.generalCancelCta] if not provided.
  final String? cancelCta;

  /// App bar (e.g. [WalletAppBar]) shown at the top of the page.
  ///
  /// Used to display the progress of a multi-step flow if provided.
  final PreferredSizeWidget? appBar;

  /// Optional contextual image (e.g., a logo) displayed above the title.
  ///
  /// Typically implemented as [Image.asset(WalletAssets.logo_wallet, height: 64, width: 64)].
  final Widget? contextImage;

  /// The loading indicator widget that shows progress during the operation.
  ///
  /// Defaults to an inset [LoadingIndicator] but can be customized if needed.
  final Widget loadingIndicator;

  /// Controls whether the title automatically requests accessibility focus, and whether a title that changes while
  /// the page stays on screen is announced.
  ///
  /// Set to false only if the page has its own focus management.
  final bool requestAccessibilityFocus;

  const GenericLoadingPage({
    required this.title,
    this.description,
    this.onCancel,
    this.cancelCta,
    this.appBar,
    this.requestAccessibilityFocus = true,
    this.contextImage,
    this.loadingIndicator = const Padding(padding: EdgeInsets.all(16), child: LoadingIndicator()),
    super.key,
  });

  @override
  State<GenericLoadingPage> createState() => _GenericLoadingPageState();
}

class _GenericLoadingPageState extends State<GenericLoadingPage> {
  final GlobalKey _titleKey = GlobalKey();

  @override
  void initState() {
    super.initState();
    if (widget.requestAccessibilityFocus) {
      // Using addPostFrameCallback because changing focus need to wait for the widget to finish rendering.
      WidgetsBinding.instance.addPostFrameCallback((_) async {
        /// Because [GenericLoadingPage] often lives within a [FakePagingAnimatedSwitcher]. We delay moving the focus by
        /// an extra [kDefaultAnimationDuration] so that any animations can settle (this allows the focus change
        /// to behave properly on iOS as well).
        if (!Environment.isTest) await Future.delayed(kDefaultAnimationDuration);
        if (!mounted) return;
        _titleKey.currentContext?.findRenderObject()?.sendSemanticsEvent(const FocusSemanticEvent());
      });
    }
  }

  @override
  void didUpdateWidget(covariant GenericLoadingPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    // A flow can swap the title of a still-mounted page (e.g. 'Please wait' -> 'Waiting for your data...'). The
    // focus request above only fires on mount, so without this the new status would go unnoticed by screen readers.
    // An announcement rather than a live region: a live region also speaks on mount, doubling the focus read-out.
    if (widget.requestAccessibilityFocus && widget.title != oldWidget.title) {
      context.read<AnnouncementService>().announce(widget.title);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: widget.appBar,
      body: SizedBox(
        width: double.infinity,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Expanded(
              flex: context.isLandscape ? 3 : 2,
              child: SingleChildScrollView(
                reverse: true,
                /* make sure it's bottom aligned */
                padding: const EdgeInsets.symmetric(horizontal: _kHorizontalPadding),
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.end,
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    widget.contextImage ?? const SizedBox.shrink(),
                    SizedBox(height: widget.contextImage == null ? 0 : _kContextLogoSpacing),
                    TitleText(
                      widget.title,
                      style: BaseWalletTheme.headlineExtraSmallTextStyle,
                      textAlign: TextAlign.center,
                      key: _titleKey,
                    ),
                    if (widget.description != null) ...[
                      const SizedBox(height: 8),
                      Text(
                        widget.description!,
                        style: context.textTheme.bodyLarge,
                        textAlign: TextAlign.center,
                      ),
                    ],
                    const SizedBox(height: 24),
                  ],
                ),
              ),
            ),
            widget.loadingIndicator,
            Expanded(
              flex: 2,
              child: Column(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  _buildOptionalCancelButton(context),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildOptionalCancelButton(BuildContext context) {
    if (widget.onCancel == null) return const SizedBox.shrink();
    return SafeArea(
      left: false,
      right: false,
      child: ListButton(
        icon: const Icon(Icons.block_outlined),
        onPressed: widget.onCancel,
        dividerSide: DividerSide.top,
        mainAxisAlignment: MainAxisAlignment.center,
        text: Text(widget.cancelCta ?? context.l10n.generalCancelCta),
      ),
    );
  }
}
