import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../theme/base_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../decoration/shadow_decoration.dart';
import '../button/button_content.dart';
import '../default_text_and_focus_style.dart';
import 'wallet_card_item.dart';

const _kCornerRadius = Radius.circular(12);
const _kContentPadding = EdgeInsets.symmetric(horizontal: 24, vertical: 20);
const _kHeaderStripHeight = 40.0;
const _kChangeCardButtonMinHeight = 76.0;

/// A Card like component that lists all the titles of the provided [attributes].
/// Used in e.g. the disclosure flow to show which attributes can be shared.
class SharedAttributesCard extends StatefulWidget {
  /// The [WalletCard] represented by this widget, used to display card-specific
  /// information such as title and background.
  final WalletCard card;

  /// Optional list of [DataAttribute]s to be displayed within the card. If null,
  /// the widget will fallback to using the [card] property's own attributes.
  final List<DataAttribute>? attributes;

  /// Callback invoked when the main card area is pressed. Used to handle user
  /// interactions like navigating to a detail view.
  final VoidCallback? onPressed;

  /// Callback invoked when the "Change card" call-to-action is pressed. Used to
  /// trigger card selection workflow.
  final VoidCallback? onChangeCardPressed;

  const SharedAttributesCard({
    required this.card,
    this.attributes,
    this.onPressed,
    this.onChangeCardPressed,
    super.key,
  });

  @override
  State<SharedAttributesCard> createState() => _SharedAttributesCardState();
}

class _SharedAttributesCardState extends State<SharedAttributesCard> {
  late WidgetStatesController _statesController;

  /// Determines whether the "Change card" call-to-action button is shown.
  bool get showChangeCardButton => widget.onChangeCardPressed != null;

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (context.mounted) _statesController.addListener(() => setState(() {}));
    });
  }

  @override
  void dispose() {
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: const CardShadowDecoration(),
      child: Column(
        children: [
          TextButton(
            onPressed: widget.onPressed,
            statesController: _statesController,
            style: context.theme.textButtonTheme.style?.copyWith(
              padding: const WidgetStatePropertyAll(EdgeInsets.zero),
              shape: WidgetStatePropertyAll(_buildMainShape(context)),
              side: _resolveBorderSide(context),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildHeaderStrip(context),
                Padding(
                  padding: _kContentPadding,
                  child: _buildCardContent(context),
                ),
              ],
            ),
          ),
          showChangeCardButton ? _buildChangeCardCta(context) : const SizedBox.shrink(),
        ],
      ),
    );
  }

  /// Creates the rounded rectangle shape with a border for the main card.
  /// When [showChangeCardButton] is true, only the top corners are rounded to allow
  /// the change card button at the bottom to provide its own rounded corners.
  OutlinedBorder _buildMainShape(BuildContext context) {
    return RoundedRectangleBorder(
      side: _buildBorderSide(context),
      borderRadius: showChangeCardButton
          ? const BorderRadius.vertical(top: _kCornerRadius)
          : const BorderRadius.all(_kCornerRadius),
    );
  }

  /// Builds the content section of the card, including the title, attributes,
  /// and the "View" call-to-action button.
  ///
  /// The content is structured as:
  /// 1. Card title (localized) with accessibility support
  /// 2. List of localized attribute labels
  /// 3. Action button with forward icon for navigation
  ///
  /// Uses [DefaultTextAndFocusStyle] for consistent text styling and focus states.
  Widget _buildCardContent(BuildContext context) {
    final textPressedColor = context.theme.textButtonTheme.style?.foregroundColor?.resolve({WidgetState.pressed});
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        DefaultTextAndFocusStyle(
          statesController: _statesController,
          textStyle: context.textTheme.titleMedium,
          pressedOrFocusedColor: textPressedColor,
          child: Text.rich(
            context.l10n
                .sharedAttributesCardTitle(
                  widget.card.title.l10nValue(context),
                  (widget.attributes ?? widget.card.attributes).length,
                )
                .toTextSpan(context),
          ),
        ),
        const SizedBox(height: 8),
        DefaultTextAndFocusStyle(
          statesController: _statesController,
          textStyle: context.textTheme.bodyLarge,
          pressedOrFocusedColor: textPressedColor,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: _buildAttributeList(context),
          ),
        ),
        const SizedBox(height: 16),
        Focus(
          // Prevents the button from being focused to avoid duplicate focus handling with the parent TextButton
          canRequestFocus: false,
          descendantsAreFocusable: false,
          child: TextButton.icon(
            onPressed: widget.onPressed,
            statesController: _statesController,
            icon: const Icon(Icons.arrow_forward),
            iconAlignment: IconAlignment.end,
            label: Semantics(
              button: true,
              attributedLabel: context.l10n
                  .sharedAttributesCardCtaSemanticsLabel(widget.card.title.l10nValue(context))
                  .toAttributedString(context),
              excludeSemantics: true /* exclude semantics from child text */,
              child: Text.rich(context.l10n.sharedAttributesCardCta.toTextSpan(context)),
            ),
            style: context.theme.textButtonTheme.style?.copyWith(
              backgroundColor: WidgetStateProperty.all(Colors.transparent),
              minimumSize: const WidgetStatePropertyAll(Size.zero),
              padding: const WidgetStatePropertyAll(EdgeInsets.zero),
              side: WidgetStateBorderSide.resolveWith((states) => BorderSide.none),
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
          ),
        ),
        const SizedBox(height: 4),
      ],
    );
  }

  /// Builds the header section of the card, displaying a localized background
  /// for the associated [WalletCard]. The header has a fixed height and
  /// rounded top corners to match the card's visual design.
  Widget _buildHeaderStrip(BuildContext context) {
    return SizedBox(
      height: _kHeaderStripHeight,
      width: double.infinity,
      child: ClipRRect(
        borderRadius: const BorderRadius.only(
          topLeft: _kCornerRadius,
          topRight: _kCornerRadius,
        ),
        child: widget.card.getL10nBackground(context),
      ),
    );
  }

  List<Widget> _buildAttributeList(BuildContext context) => (widget.attributes ?? widget.card.attributes)
      .map((attribute) => Text.rich(attribute.label.l10nSpan(context)))
      .toList();

  /// Builds the "Change card" call-to-action (CTA) button, used at the bottom of the card.
  Widget _buildChangeCardCta(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      child: TextButton(
        style: context.theme.textButtonTheme.style?.copyWith(
          alignment: Alignment.centerLeft,
          minimumSize: const WidgetStatePropertyAll(Size(0, _kChangeCardButtonMinHeight)),
          padding: const WidgetStatePropertyAll(EdgeInsets.symmetric(horizontal: 24)),
          side: _resolveBorderSide(context),
          shape: WidgetStateOutlinedBorder.resolveWith(
            (states) => const RoundedRectangleBorder(
              borderRadius: BorderRadiusGeometry.vertical(bottom: _kCornerRadius),
            ),
          ),
        ),
        onPressed: widget.onChangeCardPressed,
        child: ButtonContent(
          text: Text(context.l10n.sharedAttributesCardChangeCardCta),
          icon: const Icon(Icons.credit_card_outlined),
          iconPosition: IconPosition.end,
          mainAxisAlignment: MainAxisAlignment.start,
        ),
      ),
    );
  }

  WidgetStateBorderSide _resolveBorderSide(BuildContext context) {
    return WidgetStateBorderSide.resolveWith(
      (states) {
        return states.isPressedOrFocused
            ? null /* default behaviour */
            : _buildBorderSide(context);
      },
    );
  }

  /// Creates a standard border side used for the un-focused state of the card components.
  BorderSide _buildBorderSide(BuildContext context) => BorderSide(
        color: context.colorScheme.outlineVariant,
        strokeAlign: BorderSide.strokeAlignOutside,
        width: 1,
      );
}
