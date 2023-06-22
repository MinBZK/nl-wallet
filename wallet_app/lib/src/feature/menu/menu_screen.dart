import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/animated_visibility_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import 'bloc/menu_bloc.dart';
import 'page/menu_about_page.dart';
import 'page/menu_main_page.dart';
import 'page/menu_settings_page.dart';

class MenuScreen extends StatelessWidget {
  const MenuScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: _buildBackButton(context),
        title: _buildTitle(context),
      ),
      body: _buildBody(),
    );
  }

  Widget _buildTitle(BuildContext context) {
    return BlocBuilder<MenuBloc, MenuState>(
      builder: (context, state) {
        String title = context.l10n.menuScreenMainTitle;
        if (state is MenuLoadSuccess) {
          switch (state.menu) {
            case SelectedMenu.main:
              title = context.l10n.menuScreenMainTitle;
              break;
            case SelectedMenu.settings:
              title = context.l10n.menuScreenSettingsTitle;
              break;
            case SelectedMenu.about:
              title = context.l10n.menuScreenAboutTitle;
              break;
          }
        }
        return Text(title);
      },
    );
  }

  Widget _buildBackButton(BuildContext context) {
    return BlocBuilder<MenuBloc, MenuState>(
      builder: (context, state) {
        return AnimatedVisibilityBackButton(
          visible: (state is MenuLoadSuccess && state.menu != SelectedMenu.main),
          onPressed: () => context.read<MenuBloc>().add(MenuBackPressed()),
        );
      },
    );
  }

  Widget _buildBody() {
    return BlocBuilder<MenuBloc, MenuState>(
      builder: (context, state) {
        if (state is MenuInitial) return const CenteredLoadingIndicator();
        if (state is MenuLoadInProgress) return const CenteredLoadingIndicator();
        if (state is MenuLoadSuccess) return _buildSuccess(context, state);
        throw UnsupportedError('Unknown state: ${state.runtimeType}');
      },
    );
  }

  Widget _buildSuccess(BuildContext context, MenuLoadSuccess state) {
    switch (state.menu) {
      case SelectedMenu.main:
        return MenuMainPage(name: state.name);
      case SelectedMenu.settings:
        return const MenuSettingsPage();
      case SelectedMenu.about:
        return const MenuAboutPage();
    }
  }
}
