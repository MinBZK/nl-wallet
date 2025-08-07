import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/card/wallet_card.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../common/sheet/select_card_sheet.dart';
import '../common/widget/animation/animated_card_switcher.dart';
import '../common/widget/attribute/attribute_row.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/button/secondary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/card/wallet_card_item.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/spacer/sliver_divider.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'bloc/check_attributes_bloc.dart';

/// A screen to display and verify the attributes of a selected wallet card.
///
/// This screen is intended to be launched using either the [show] or [showWithAlternatives] factory methods.
/// - [show] is used when displaying attributes for a single card (no alternatives).
/// - [showWithAlternatives] is used when the user can switch between multiple cards.
///
/// The screen displays a list of attributes, a "data incorrect" CTA, and optionally a "change card" CTA
/// if alternatives are available.
class CheckAttributesScreen extends StatelessWidget {
  /// Called when the user presses the "data incorrect" button.
  final VoidCallback? onDataIncorrectPressed;

  /// Called when an alternative card is selected from the selection sheet.
  /// [WalletCard] argument represents the selected card.
  final Function(WalletCard)? onAlternativeCardSelected;

  const CheckAttributesScreen({
    this.onDataIncorrectPressed,
    this.onAlternativeCardSelected,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        actions: const [HelpIconButton()],
        title: FadeInAtOffset(
          appearOffset: 120,
          visibleOffset: 150,
          child: TitleText(
            _generateTitle(
              context,
              tryCast(context.read<CheckAttributesBloc>().state),
            ),
          ),
        ),
      ),
      body: SafeArea(
        child: BlocBuilder<CheckAttributesBloc, CheckAttributesState>(
          builder: (context, state) {
            switch (state) {
              case CheckAttributesInitial():
                return _buildLoading();
              case CheckAttributesSuccess():
                return _buildSuccess(context, state);
            }
          },
        ),
      ),
    );
  }

  Widget _buildLoading() {
    return const Column(
      children: [
        Expanded(child: CenteredLoadingIndicator()),
        BottomBackButton(),
      ],
    );
  }

  Widget _buildSuccess(BuildContext context, CheckAttributesSuccess state) {
    return Column(
      children: [
        Expanded(
          child: WalletScrollbar(
            child: AnimatedCardSwitcher(
              enableAnimation: !context.isScreenReaderEnabled,
              child: CustomScrollView(
                key: ValueKey<int>(state.card.hashCode),
                slivers: [
                  SliverToBoxAdapter(
                    child: Container(
                      padding: const EdgeInsets.all(16),
                      alignment: AlignmentDirectional.centerStart,
                      child: SizedBox(
                        width: 110,
                        child: ExcludeSemantics(
                          child: WalletCardItem.fromWalletCard(
                            context,
                            state.card,
                            scaleText: false,
                            showText: false,
                          ),
                        ),
                      ),
                    ),
                  ),
                  SliverToBoxAdapter(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          TitleText(_generateTitle(context, state)),
                          const SizedBox(height: 8),
                          BlocBuilder<CheckAttributesBloc, CheckAttributesState>(
                            builder: (context, state) {
                              switch (state) {
                                case CheckAttributesInitial():
                                  return const SizedBox.shrink();
                                case CheckAttributesSuccess():
                                  return SizedBox(
                                    width: double.infinity,
                                    child: BodyText(
                                      context.l10n.checkAttributesScreenSubtitle(
                                        state.card.issuer.displayName.l10nValue(context),
                                      ),
                                      textAlign: TextAlign.start,
                                    ),
                                  );
                              }
                            },
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SliverSizedBox(height: 24),
                  const SliverDivider(),
                  SliverPadding(
                    padding: const EdgeInsets.symmetric(vertical: 24),
                    sliver: SliverList.separated(
                      itemCount: state.attributes.length,
                      itemBuilder: (context, i) {
                        final attribute = state.attributes[i];
                        return AttributeRow(attribute: attribute);
                      },
                      separatorBuilder: (context, i) => const SizedBox(height: 24),
                    ),
                  ),
                  SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
                  const SliverSizedBox(height: 32),
                ],
              ),
            ),
          ),
        ),
        _buildBottomSection(context, state),
      ],
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    if (onDataIncorrectPressed == null) return const SizedBox.shrink();
    return ListButton(
      onPressed: onDataIncorrectPressed,
      text: Text.rich(context.l10n.checkAttributesScreenDataIncorrectCta.toTextSpan(context)),
    );
  }

  String _generateTitle(BuildContext context, CheckAttributesSuccess? state) {
    if (state == null) return '';
    return context.l10n.checkAttributesScreenTitle(
      state.attributes.length,
      state.attributes.length,
      state.card.title.l10nValue(context),
    );
  }

  Widget _buildBottomSection(BuildContext context, CheckAttributesSuccess state) {
    if (onAlternativeCardSelected == null || !state.showChangeCardCta) return const BottomBackButton();
    return Column(
      children: [
        const Divider(),
        ConfirmButtons(
          secondaryButton: TertiaryButton(
            text: Text(context.l10n.generalBottomBackCta),
            icon: const Icon(Icons.arrow_back_outlined),
            onPressed: () => Navigator.pop(context),
          ),
          primaryButton: SecondaryButton(
            text: Text(context.l10n.checkAttributesScreenChangeCardCta),
            icon: const Icon(Icons.credit_card_outlined),
            onPressed: () async {
              final result = await SelectCardSheet.show(context, candidates: state.alternatives ?? []);
              if (result != null && context.mounted) {
                context.read<CheckAttributesBloc>().add(CheckAttributesCardSelected(card: result));
                onAlternativeCardSelected?.call(result);
              }
            },
          ),
        ),
      ],
    );
  }

  /// Displays this screen for a single [WalletCard], using [attributes] if provided
  /// or falling back to the card's own attributes if none are specified.
  static void show(
    BuildContext context, {
    required WalletCard card,
    List<DataAttribute>? attributes,
    VoidCallback? onDataIncorrectPressed,
  }) {
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) {
          return BlocProvider<CheckAttributesBloc>(
            create: (context) => CheckAttributesBloc.forCard(
              card,
              attributes: attributes,
            ),
            child: CheckAttributesScreen(
              onDataIncorrectPressed: onDataIncorrectPressed,
            ),
          );
        },
      ),
    );
  }

  /// Displays the check attributes screen with a selected [WalletCard] and a list of available [cards] to switch between.
  ///
  /// This method is used when the user can select from multiple cards.
  /// The selected card (`selection`) is displayed, and the callback [onAlternativeCardSelected] is triggered
  /// when the user navigates to a different card from the provided list.
  ///
  /// The [cards] list must include the [selection] card.
  /// [onAlternativeCardSelected] is required to handle navigation/refresh when a new card is selected.
  /// For single-card displays (no switching options), use [show] instead.
  static void showWithAlternatives(
    BuildContext context, {
    required WalletCard selection,
    required List<WalletCard> cards,
    required Function(WalletCard) onAlternativeCardSelected,
    VoidCallback? onDataIncorrectPressed,
  }) {
    assert(cards.contains(selection), 'The cards list should include the selected card.');
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) {
          return BlocProvider<CheckAttributesBloc>(
            create: (context) => CheckAttributesBloc(cards: cards)..add(CheckAttributesCardSelected(card: selection)),
            child: CheckAttributesScreen(
              onDataIncorrectPressed: onDataIncorrectPressed,
              onAlternativeCardSelected: onAlternativeCardSelected,
            ),
          );
        },
      ),
    );
  }
}
