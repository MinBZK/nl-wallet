import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/timeline_attribute.dart';
import '../../../../domain/usecase/wallet/get_wallet_timeline_attributes_usecase.dart';

part 'wallet_history_event.dart';
part 'wallet_history_state.dart';

class WalletHistoryBloc extends Bloc<WalletHistoryEvent, WalletHistoryState> {
  final GetWalletTimelineAttributesUseCase getWalletTimelineAttributesUseCase;

  WalletHistoryBloc(
    this.getWalletTimelineAttributesUseCase,
  ) : super(WalletHistoryInitial()) {
    on<WalletHistoryLoadTriggered>(_onWalletHistoryLoadTriggered);

    add(const WalletHistoryLoadTriggered());
  }

  void _onWalletHistoryLoadTriggered(WalletHistoryLoadTriggered event, emit) async {
    emit(const WalletHistoryLoadInProgress());
    try {
      List<TimelineAttribute> attributes = await getWalletTimelineAttributesUseCase.invoke();
      emit(WalletHistoryLoadSuccess(attributes));
    } catch (error) {
      emit(const WalletHistoryLoadFailure());
    }
  }
}
