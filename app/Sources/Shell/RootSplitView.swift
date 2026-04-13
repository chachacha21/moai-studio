//
//  RootSplitView.swift
//  NavigationSplitView 기반 사이드바 + 콘텐츠 영역 레이아웃 (RG-M1-1, RG-M1-6).
//
//  @MX:NOTE: [AUTO] 사이드바 너비는 navigationSplitViewColumnWidth(min:200, ideal:250, max:400) 로 클램프.
//  @MX:NOTE: [AUTO] MS-2 (T-043): detail 영역을 ContentArea → PaneContainer 로 교체.
//             ContentArea.swift 는 MS-3 에서 TabBarView 통합 완료 후 제거 예정.
//

import SwiftUI

struct RootSplitView: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    @Environment(WindowStateStore.self) private var windowState

    var body: some View {
        @Bindable var viewModelBindable = viewModel

        NavigationSplitView {
            WorkspaceListView()
                .navigationSplitViewColumnWidth(
                    min: WindowStateStore.sidebarMinWidth,
                    ideal: windowState.sidebarWidth,
                    max: WindowStateStore.sidebarMaxWidth
                )
        } detail: {
            // MS-2 T-043: PaneContainer 가 PaneSplitView binary tree 를 렌더링한다.
            // 워크스페이스 미선택 시 EmptyState 를 표시한다.
            PaneContainer(selectedWorkspaceId: viewModelBindable.selectedWorkspaceId)
        }
        .navigationSplitViewStyle(.balanced)
    }
}
