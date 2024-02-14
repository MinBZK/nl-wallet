import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/sheet/explanation_sheet.dart';
import '../../common/widget/attribute/data_attribute_row.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/wallet_app_bar.dart';
import 'argument/card_data_screen_argument.dart';
import 'bloc/card_data_bloc.dart';
import 'card_data_incorrect_screen.dart';
import 'widget/data_privacy_banner.dart';

@visibleForTesting
const kPrivacyBannerKey = Key('privacyBanner');

class CardDataScreen extends StatelessWidget {
  static CardDataScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return CardDataScreenArgument.fromMap(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [CardDataScreenArgument] when opening the CardDataScreen');
    }
  }

  final String cardTitle;

  const CardDataScreen({required this.cardTitle, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('cardDataScreen'),
      appBar: _buildAppBar(context),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context) {
    final fallbackAppBarTitleText = Text(cardTitle);
    return WalletAppBar(
      title: BlocBuilder<CardDataBloc, CardDataState>(
        builder: (context, state) {
          return switch (state) {
            CardDataInitial() => fallbackAppBarTitleText,
            CardDataLoadInProgress() => fallbackAppBarTitleText,
            CardDataLoadSuccess() => Text(state.card.front.title.l10nValue(context)),
            CardDataLoadFailure() => fallbackAppBarTitleText,
          };
        },
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return Column(
      children: [
        DataPrivacyBanner(
          onPressed: () => _showDataPrivacySheet(context),
          key: kPrivacyBannerKey,
        ),
        Expanded(
          child: BlocBuilder<CardDataBloc, CardDataState>(
            builder: (context, state) {
              return switch (state) {
                CardDataInitial() => _buildLoading(),
                CardDataLoadInProgress() => _buildLoading(),
                CardDataLoadSuccess() => _buildDataAttributes(context, state.card.attributes),
                CardDataLoadFailure() => _buildError(context),
              };
            },
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildDataAttributes(BuildContext context, List<DataAttribute> attributes) {
    final List<Widget> slivers = [];

    // Data attributes
    slivers.add(const SliverSizedBox(height: 24));
    for (var element in attributes) {
      slivers.add(SliverToBoxAdapter(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: DataAttributeRow(attribute: element),
        ),
      ));
    }

    // Incorrect button
    slivers.add(const SliverToBoxAdapter(child: Divider(height: 32)));
    slivers.add(SliverToBoxAdapter(child: _buildIncorrectButton(context)));
    slivers.add(const SliverSizedBox(height: 16));
    slivers.add(const SliverToBoxAdapter(child: Divider(height: 1)));

    return Scrollbar(
      child: CustomScrollView(
        slivers: slivers,
      ),
    );
  }

  Widget _buildIncorrectButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: LinkButton(
          child: Text(context.l10n.cardDataScreenIncorrectCta),
          onPressed: () => CardDataIncorrectScreen.show(context),
        ),
      ),
    );
  }

  void _showDataPrivacySheet(BuildContext context) {
    ExplanationSheet.show(
      context,
      title: context.l10n.cardDataScreenDataPrivacySheetTitle,
      description: context.l10n.cardDataScreenDataPrivacySheetDescription,
      closeButtonText: context.l10n.generalSheetCloseCta,
    );
  }

  Widget _buildError(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Spacer(),
          Text(
            context.l10n.errorScreenGenericDescription,
            textAlign: TextAlign.center,
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: () {
              final settings = ModalRoute.of(context)?.settings;
              if (settings != null) {
                final args = getArgument(settings);
                context.read<CardDataBloc>().add(CardDataLoadTriggered(args.cardId));
              } else {
                Navigator.pop(context);
              }
            },
            child: Text(context.l10n.generalRetry),
          ),
        ],
      ),
    );
  }
}
