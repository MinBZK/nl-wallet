import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/help/help_category.dart';
import '../../domain/model/result/application_error.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../error/error_page.dart';
import 'bloc/help_overview_bloc.dart';
import 'extension/help_categories_extension.dart';

class HelpOverviewScreen extends StatelessWidget {
  const HelpOverviewScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('helpOverviewScreen'),
      appBar: WalletAppBar(title: TitleText(context.l10n.menuScreenHelpCta)),
      body: BlocBuilder<HelpOverviewBloc, HelpOverviewState>(
        builder: (context, state) => switch (state) {
          HelpOverviewInitial() || HelpOverviewLoadInProgress() => _buildScaffoldBody(_buildLoading()),
          HelpOverviewLoadSuccess() => _buildScaffoldBody(_buildContent(context, state.categories)),
          HelpOverviewLoadFailure() => _buildError(context, state.error),
        },
      ),
    );
  }

  Widget _buildScaffoldBody(Widget child) {
    return SafeArea(
      child: Column(
        children: [
          Expanded(child: child),
          const BottomBackButton(),
        ],
      ),
    );
  }

  Widget _buildError(BuildContext context, ApplicationError error) {
    return ErrorPage.fromError(
      context,
      error,
      onPrimaryActionPressed: () => Navigator.pop(context),
      style: .close,
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildContent(BuildContext context, List<HelpCategory> categories) {
    return WalletScrollbar(
      child: ListView(
        children: [
          Padding(
            padding: kDefaultTitlePadding,
            child: TitleText(context.l10n.menuScreenHelpCta),
          ),
          const SizedBox(height: 16),
          const Divider(),
          _buildTourItem(context),
          const Divider(),
          for (final category in categories) ...[
            MenuItem(
              label: Text(category.title),
              subtitle: Text(category.subtitle),
              leftIcon: Icon(category.iconData),
              onPressed: () => Navigator.pushNamed(
                context,
                WalletRoutes.helpCategoryRoute,
                arguments: category,
              ),
            ),
            const Divider(),
          ],
          _buildContactItem(context),
        ],
      ),
    );
  }

  MenuItem _buildTourItem(BuildContext context) => MenuItem(
    label: Text.rich(context.l10n.menuScreenTourCta.toTextSpan(context)),
    leftIcon: const Icon(Icons.play_arrow),
    onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.tourOverviewRoute),
  );

  MenuItem _buildContactItem(BuildContext context) => MenuItem(
    label: Text(context.l10n.contactScreenTitle),
    leftIcon: const Icon(Icons.headset_mic_outlined),
    onPressed: () => Navigator.pushNamed(context, WalletRoutes.contactRoute),
  );
}
