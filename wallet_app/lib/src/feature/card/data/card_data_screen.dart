import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../util/cast_util.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/attribute/data_attribute_row.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
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

  const CardDataScreen({required this.cardTitle, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('cardDataScreen'),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  String _generateTitle(BuildContext context, CardDataState state) {
    final title = tryCast<CardDataLoadSuccess>(state)?.card.front.title.l10nValue(context) ?? cardTitle;
    return context.l10n.cardDataScreenTitle(title);
  }

  Widget _buildBody(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: BlocBuilder<CardDataBloc, CardDataState>(
            builder: (context, state) {
              List<Widget> contentSlivers = switch (state) {
                CardDataInitial() => _buildLoading(),
                CardDataLoadInProgress() => _buildLoading(),
                CardDataLoadSuccess() => _buildDataAttributes(context, state.card.attributes),
                CardDataLoadFailure() => _buildError(context),
              };
              return CustomScrollView(
                slivers: [
                  SliverWalletAppBar(
                    title: _generateTitle(context, state),
                  ),
                  const SliverToBoxAdapter(
                    child: Padding(
                      padding: EdgeInsets.symmetric(horizontal: 16),
                      child: DataPrivacyBanner(key: kPrivacyBannerKey),
                    ),
                  ),
                  ...contentSlivers,
                ],
              );
            },
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  List<Widget> _buildLoading() {
    return [
      const SliverFillRemaining(
        child: CenteredLoadingIndicator(),
      ),
    ];
  }

  List<Widget> _buildDataAttributes(BuildContext context, List<DataAttribute> attributes) {
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
    slivers.add(const SliverSizedBox(height: 24));

    return slivers;
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

  List<Widget> _buildError(BuildContext context) {
    return [
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
        fillOverscroll: false,
        hasScrollBody: false,
        child: Align(
          alignment: Alignment.bottomCenter,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            child: ElevatedButton(
              onPressed: () => _reloadCardData(context),
              child: Text(context.l10n.generalRetry),
            ),
          ),
        ),
      ),
    ];
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
