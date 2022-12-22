import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../theme/wallet_theme_light_constants.dart';
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
  const MockDigidScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: Theme.of(context).copyWith(
        primaryColor: _kDigidOrange,
        colorScheme: WalletThemeConstants.colorScheme.copyWith(primary: _kDigidOrange),
        outlinedButtonTheme: outlinedButtonTheme(context),
      ),
      child: BlocConsumer<MockDigidBloc, MockDigidState>(
        listener: (context, state) {
          if (state is MockDigidLoggedIn) Navigator.pop(context);
        },
        builder: (context, state) {
          Widget? result;
          if (state is MockDigidInitial) result = const DigidSplashPage();
          if (state is MockDigidEnteringPin) result = _buildEnteringPinPage(state, context);
          if (state is MockDigidConfirmApp) result = _buildConfirmAppPage(context);
          if (state is MockDigidLoadInProgress) result = DigidLoadingPage(mockDelay: state.mockDelay);
          if (state is MockDigidLoggedIn) result = const DigidLoadingPage(mockDelay: Duration.zero);
          if (result == null) throw UnsupportedError('Unknown state: $state');
          return AnimatedSwitcher(duration: kDefaultAnimationDuration, child: result);
        },
      ),
    );
  }

  Widget _buildEnteringPinPage(MockDigidEnteringPin state, BuildContext context) {
    return DigidPinPage(
      selectedIndex: state.enteredDigits,
      onKeyPressed: (key) => context.read<MockDigidBloc>().add(MockDigidPinKeyPressed()),
      onBackspacePressed: () => context.read<MockDigidBloc>().add(MockDigidPinBackspacePressed()),
    );
  }

  Widget _buildConfirmAppPage(BuildContext context) {
    return DigidConfirmAppPage(onConfirmClicked: () {
      context.read<MockDigidBloc>().add(MockDigidConfirmPressed());
    });
  }

  OutlinedButtonThemeData outlinedButtonTheme(BuildContext context) {
    return OutlinedButtonThemeData(
      style: Theme.of(context).outlinedButtonTheme.style?.copyWith(
            side: const MaterialStatePropertyAll(
              BorderSide(
                color: Color(0xFFD2762B),
                width: 1,
              ),
            ),
          ),
    );
  }

  static Future<void> show(BuildContext context) {
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
