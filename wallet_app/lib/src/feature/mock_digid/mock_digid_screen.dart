// coverage:ignore-file
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/color_extension.dart';
import '../../wallet_constants.dart';
import 'bloc/mock_digid_bloc.dart';
import 'page/digid_confirm_app_page.dart';
import 'page/digid_loading_page.dart';
import 'page/digid_pin_page.dart';
import 'page/digid_splash_page.dart';

const _kDigidOrange = Color(0xFFD2762B);

/// Screen that can be navigated to when DigiD authentication is to be faked.
/// Most likely used via 'await MockDigidScreen.show(context);`
class MockDigidScreen extends StatelessWidget {
  const MockDigidScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: context.theme.copyWith(
        primaryColor: _kDigidOrange,
        colorScheme: context.colorScheme.copyWith(primary: _kDigidOrange),
        outlinedButtonTheme: outlinedButtonTheme(context),
        elevatedButtonTheme: elevatedButtonTheme(context),
      ),
      child: BlocConsumer<MockDigidBloc, MockDigidState>(
        listener: (context, state) {
          if (state is MockDigidLoggedIn) Navigator.pop(context, true);
          if (state is MockDigidRejected) Navigator.pop(context, false);
        },
        builder: (context, state) {
          final Widget result = switch (state) {
            MockDigidInitial() => const DigidSplashPage(),
            MockDigidEnteringPin() => _buildEnteringPinPage(context, state),
            MockDigidConfirmApp() => _buildConfirmAppPage(context),
            MockDigidLoadInProgress() => DigidLoadingPage(mockDelay: state.mockDelay),
            MockDigidLoggedIn() => const DigidLoadingPage(mockDelay: Duration.zero),
            MockDigidRejected() => const DigidLoadingPage(mockDelay: Duration.zero),
          };
          return AnimatedSwitcher(duration: kDefaultAnimationDuration, child: result);
        },
      ),
    );
  }

  Widget _buildEnteringPinPage(BuildContext context, MockDigidEnteringPin state) {
    return DigidPinPage(
      selectedIndex: state.enteredDigits,
      onKeyPressed: (key) => context.read<MockDigidBloc>().add(MockDigidPinKeyPressed()),
      onBackspacePressed: () => context.read<MockDigidBloc>().add(MockDigidPinBackspacePressed()),
    );
  }

  Widget _buildConfirmAppPage(BuildContext context) {
    return DigidConfirmAppPage(
      onConfirmPressed: () {
        context.read<MockDigidBloc>().add(MockDigidConfirmPressed());
      },
      onDeclinePressed: () {
        context.read<MockDigidBloc>().add(MockDigidDeclinePressed());
      },
    );
  }

  OutlinedButtonThemeData outlinedButtonTheme(BuildContext context) {
    return OutlinedButtonThemeData(
      style: context.theme.outlinedButtonTheme.style?.copyWith(
        side: const WidgetStatePropertyAll(
          BorderSide(
            color: Color(0xFFD2762B),
            width: 1,
          ),
        ),
      ),
    );
  }

  ElevatedButtonThemeData elevatedButtonTheme(BuildContext context) {
    return ElevatedButtonThemeData(
      style: context.theme.elevatedButtonTheme.style?.copyWith(
        backgroundColor: const WidgetStatePropertyAll(_kDigidOrange),
        overlayColor: WidgetStatePropertyAll(_kDigidOrange.darken()),
      ),
    );
  }

  static Future<bool?> mockLogin(BuildContext context) {
    return Navigator.of(context).push(
      CupertinoPageRoute(
        builder: (context) => BlocProvider(
          create: (context) => MockDigidBloc(),
          child: const MockDigidScreen(),
        ),
      ),
    );
  }
}
