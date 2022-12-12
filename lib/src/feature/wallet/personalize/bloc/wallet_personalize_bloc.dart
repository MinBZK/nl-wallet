import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/card_front.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/card/get_pid_card_usecase.dart';
import '../../../../domain/usecase/card/wallet_add_issued_card_usecase.dart';

part 'wallet_personalize_event.dart';
part 'wallet_personalize_state.dart';

class WalletPersonalizeBloc extends Bloc<WalletPersonalizeEvent, WalletPersonalizeState> {
  final GetPidCardUseCase getPidCardUseCase;
  final WalletAddIssuedCardUseCase walletAddIssuedCardUseCase;

  WalletPersonalizeBloc(this.getPidCardUseCase, this.walletAddIssuedCardUseCase) : super(WalletPersonalizeInitial()) {
    on<WalletPersonalizeLoginWithDigidClicked>(_onLoginWithDigidClicked);
    on<WalletPersonalizeLoginWithDigidSucceeded>(_onLoginWithDigidSucceeded);
    on<WalletPersonalizeOfferingVerified>(_onOfferingVerified);
    on<WalletPersonalizeScanInitiated>(_onScanInitiated);
    on<WalletPersonalizeScanEvent>(_onScanEvent);
    on<WalletPersonalizePhotoApproved>(_onPhotoApproved);
    on<WalletPersonalizeOnRetryClicked>(_onRetryClicked);
    on<WalletPersonalizeOnBackPressed>(_onBackPressed);
  }

  void _onLoginWithDigidClicked(event, emit) async {
    emit(WalletPersonalizeLoadingPid());
  }

  void _onLoginWithDigidSucceeded(event, emit) async {
    try {
      final card = await getPidCardUseCase.invoke();
      // For brevity I opted to use' the getPidCardUseCase here to get the data, that way we rely on the same mock data.
      final firstNames = card.attributes.firstWhere((element) => element.type == AttributeType.firstNames).value;
      final availableAttributes = card.attributes.where((element) => element.type != AttributeType.profilePhoto);
      emit(WalletPersonalizeCheckData(firstNames: firstNames, availableAttributes: availableAttributes.toList()));
    } catch (ex, stack) {
      Fimber.e('Failed to get PID', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  void _onOfferingVerified(WalletPersonalizeOfferingVerified event, emit) async {
    emit(const WalletPersonalizeScanIdIntro());
  }

  void _onScanInitiated(event, emit) async {
    emit(WalletPersonalizeScanId());
  }

  void _onScanEvent(event, emit) async {
    const mockDelay = Duration(seconds: 4);
    emit(const WalletPersonalizeLoadingPhoto(mockDelay));
    await Future.delayed(mockDelay);
    try {
      final card = await getPidCardUseCase.invoke();
      final photo = card.attributes.firstWhere((element) => element.type == AttributeType.profilePhoto);
      emit(WalletPersonalizePhotoAdded(photo));
    } catch (ex, stack) {
      Fimber.e('Failed create find photo', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  void _onPhotoApproved(event, emit) async {
    try {
      final card = await getPidCardUseCase.invoke();
      await walletAddIssuedCardUseCase.invoke(card);
      emit(WalletPersonalizeSuccess(card));
    } catch (ex, stack) {
      Fimber.e('Failed create PID card', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  void _onRetryClicked(event, emit) async {
    emit(WalletPersonalizeInitial());
  }

  void _onBackPressed(event, emit) async {
    if (state.canGoBack) {
      if (state is WalletPersonalizeScanId) {
        emit(const WalletPersonalizeScanIdIntro(afterBackPressed: true));
      }
    }
  }
}
