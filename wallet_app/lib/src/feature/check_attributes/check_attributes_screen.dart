import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/card/wallet_card.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/formatter/attribute_value_formatter.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/card/wallet_card_item.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/spacer/sliver_divider.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'bloc/check_attributes_bloc.dart';

class CheckAttributesScreen extends StatelessWidget {
  final VoidCallback? onDataIncorrectPressed;

  const CheckAttributesScreen({
    this.onDataIncorrectPressed,
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
          child: Text(_generateTitle(context)),
        ),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildContent(context),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    /// Since the card & attributes don't change throughout the lifecycle of the bloc (see [CheckAttributesBloc])
    /// we can just fetch them directly (without the complexity of a separate BlocBuilder).
    final card = context.read<CheckAttributesBloc>().state.card;
    final attributes = context.read<CheckAttributesBloc>().state.attributes;
    return WalletScrollbar(
      child: CustomScrollView(
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
                    card,
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
                  TitleText(_generateTitle(context)),
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
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            sliver: SliverList.separated(
              itemCount: attributes.length,
              itemBuilder: (context, i) {
                final attribute = attributes[i];
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      attribute.label.l10nValue(context),
                      style: context.textTheme.bodySmall,
                    ),
                    Text(
                      attribute.value.prettyPrint(context),
                      style: context.textTheme.titleMedium,
                    ),
                  ],
                );
              },
              separatorBuilder: (context, i) => const SizedBox(height: 24),
            ),
          ),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverSizedBox(height: 32),
        ],
      ),
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    if (onDataIncorrectPressed == null) return const SizedBox.shrink();
    return ListButton(
      onPressed: onDataIncorrectPressed,
      text: Text.rich(context.l10n.checkAttributesScreenDataIncorrectCta.toTextSpan(context)),
    );
  }

  String _generateTitle(BuildContext context) {
    final card = context.read<CheckAttributesBloc>().state.card;
    final attributes = context.read<CheckAttributesBloc>().state.attributes;
    return context.l10n.checkAttributesScreenTitle(
      attributes.length,
      attributes.length,
      card.title.l10nValue(context),
    );
  }

  static void show(
    BuildContext context, {
    required WalletCard card,
    required List<DataAttribute> attributes,
    VoidCallback? onDataIncorrectPressed,
  }) {
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) {
          return BlocProvider<CheckAttributesBloc>(
            create: (context) => CheckAttributesBloc(
              attributes: attributes,
              card: card,
            )..add(CheckAttributesLoadTriggered()),
            child: CheckAttributesScreen(
              onDataIncorrectPressed: onDataIncorrectPressed,
            ),
          );
        },
      ),
    );
  }
}
