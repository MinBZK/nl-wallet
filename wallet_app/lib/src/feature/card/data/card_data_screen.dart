import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/cast_util.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/attribute/data_attribute_row.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';
import 'argument/card_data_screen_argument.dart';
import 'bloc/card_data_bloc.dart';
import 'card_data_incorrect_screen.dart';

class CardDataScreen extends StatelessWidget {
  static CardDataScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return CardDataScreenArgument.fromMap(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [CardDataScreenArgument] when opening the CardDataScreen');
    }
  }

  final String cardTitle;

  const CardDataScreen({required this.cardTitle, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(_generateTitle(context)),
        actions: const [HelpIconButton()],
      ),
      key: const Key('cardDataScreen'),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  String _generateTitle(BuildContext context) {
    final state = context.watch<CardDataBloc>().state;
    final title = tryCast<CardDataLoadSuccess>(state)?.card.title.l10nValue(context) ?? cardTitle;
    return context.l10n.cardDataScreenTitle(title);
  }

  Widget _buildBody(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: BlocBuilder<CardDataBloc, CardDataState>(
            builder: (context, state) {
              final Widget contentSliver = switch (state) {
                CardDataInitial() => _buildLoading(),
                CardDataLoadInProgress() => _buildLoading(),
                CardDataLoadSuccess() => _buildSuccess(context, state),
                CardDataLoadFailure() => _buildError(context),
              };
              return WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: kDefaultTitlePadding,
                        child: TitleText(_generateTitle(context)),
                      ),
                    ),
                    contentSliver,
                  ],
                ),
              );
            },
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  Widget _buildSuccess(BuildContext context, CardDataLoadSuccess state) {
    return SliverMainAxisGroup(
      slivers: [
        const SliverSizedBox(height: 16),
        const SliverDivider(),
        const SliverSizedBox(height: 24),
        _buildDataAttributes(context, state.card.attributes),
        const SliverSizedBox(height: 24),
        _buildDataIncorrectButtonSliver(context),
        const SliverSizedBox(height: 24),
      ],
    );
  }

  Widget _buildLoading() {
    return const SliverFillRemaining(
      hasScrollBody: false,
      child: CenteredLoadingIndicator(),
    );
  }

  Widget _buildDataAttributes(BuildContext context, List<DataAttribute> attributes) {
    return SliverList.separated(
      itemBuilder: (context, i) => DataAttributeRow(attribute: attributes[i]),
      separatorBuilder: (context, i) => const SizedBox(height: 24),
      itemCount: attributes.length,
    );
  }

  Widget _buildDataIncorrectButtonSliver(BuildContext context) => SliverToBoxAdapter(
        child: _buildIncorrectButton(context),
      );

  Widget _buildIncorrectButton(BuildContext context) {
    return ListButton(
      text: Text.rich(context.l10n.cardDataScreenIncorrectCta.toTextSpan(context)),
      onPressed: () => CardDataIncorrectScreen.show(context),
    );
  }

  Widget _buildError(BuildContext context) {
    return SliverMainAxisGroup(
      slivers: [
        const SliverSizedBox(height: 24),
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              context.l10n.errorScreenGenericDescription,
              style: context.textTheme.bodyLarge,
            ),
          ),
        ),
        SliverFillRemaining(
          hasScrollBody: false,
          child: Align(
            alignment: Alignment.bottomCenter,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
              child: ElevatedButton(
                onPressed: () => _reloadCardData(context),
                child: Text.rich(context.l10n.generalRetry.toTextSpan(context)),
              ),
            ),
          ),
        ),
      ],
    );
  }

  void _reloadCardData(BuildContext context) {
    final settings = ModalRoute.of(context)?.settings;
    if (settings != null) {
      final args = getArgument(settings);
      context.read<CardDataBloc>().add(CardDataLoadTriggered(args.cardId));
    } else {
      Navigator.pop(context);
    }
  }
}
