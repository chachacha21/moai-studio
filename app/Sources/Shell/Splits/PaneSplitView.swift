//
//  PaneSplitView.swift
//  NSSplitView binary tree 래퍼 (SPEC-M2-001 MS-2 T-038, T-040, T-041).
//
//  @MX:ANCHOR: [AUTO] PaneTreeModel 을 NSSplitView 계층으로 렌더링하는 유일 진입점 (fan_in>=3)
//  @MX:REASON: [AUTO] PaneContainer, RootSplitView(리팩터링), E2E 테스트 세 경로에서 사용
//
//  NSSplitView orientation 주의:
//  - .vertical   → 분할선이 수직 → 좌우 배치 → SplitKind.horizontal 에 매핑
//  - .horizontal → 분할선이 수평 → 상하 배치 → SplitKind.vertical 에 매핑

import AppKit
import SwiftUI

// MARK: - PaneSplitView

/// pane binary tree 를 재귀적으로 NSSplitView 계층으로 렌더링하는 SwiftUI 뷰.
// @MX:NOTE: [AUTO] MS-3 에서 leaf 노드 내부는 TabBarView + SurfaceProtocol 로 교체 예정.
//            현재는 pane id + "MS-3 에서 surface 연결 예정" 플레이스홀더를 표시한다.
public struct PaneSplitView: NSViewRepresentable {
    /// 렌더링할 pane 트리 모델
    @Bindable var model: PaneTreeModel
    /// 현재 활성 pane id
    @Binding var activePaneId: Int64?

    public init(model: PaneTreeModel, activePaneId: Binding<Int64?>) {
        self.model = model
        self._activePaneId = activePaneId
    }

    public func makeNSView(context: Context) -> NSView {
        makeView(for: model.rootId, context: context)
    }

    public func updateNSView(_ nsView: NSView, context: Context) {
        // 트리 변경 시 부모 뷰가 교체하므로 여기서는 no-op
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(model: model, activePaneId: $activePaneId)
    }

    // MARK: - 재귀 뷰 생성

    private func makeView(for paneId: Int64?, context: Context) -> NSView {
        guard let paneId, let node = model.nodes[paneId] else {
            return makeEmptyView()
        }

        switch node.split {
        case .leaf:
            return makeLeafView(paneId: paneId, context: context)
        case .horizontal, .vertical:
            return makeSplitView(node: node, context: context)
        }
    }

    private func makeEmptyView() -> NSView {
        let view = NSView()
        view.wantsLayer = true
        view.layer?.backgroundColor = NSColor.windowBackgroundColor.cgColor
        return view
    }

    /// leaf 노드: 플레이스홀더 뷰 (MS-3 에서 TabBarView 로 교체)
    // @MX:NOTE: [AUTO] MS-3 까지는 pane id 를 표시하는 임시 뷰. SurfaceProtocol 연결 이후 교체.
    private func makeLeafView(paneId: Int64, context: Context) -> NSView {
        let placeholder = LeafPlaceholderView(paneId: paneId, activePaneId: $activePaneId)
        let hosting = NSHostingView(rootView: placeholder)
        hosting.translatesAutoresizingMaskIntoConstraints = false
        return hosting
    }

    /// 비-leaf 노드: NSSplitView 로 두 자식을 배치
    private func makeSplitView(node: PaneNode, context: Context) -> NSSplitView {
        let splitView = MoAISplitView()
        // @MX:NOTE: [AUTO] orientation 과 split direction 의 반전 관계:
        //           horizontal split (좌우) → orientation = .vertical (수직 분할선)
        //           vertical split (상하)   → orientation = .horizontal (수평 분할선)
        splitView.isVertical = (node.split == .horizontal)
        splitView.dividerStyle = .thin
        splitView.delegate = context.coordinator

        let children = model.children(of: node.id)
        for (index, child) in children.enumerated() {
            let childView = makeView(for: child.id, context: context)
            let item = NSSplitViewItem(viewController: NSViewController())
            item.viewController.view = childView
            // @MX:NOTE: [AUTO] 최소 pane 크기 200pt — AC-1.3 요구사항
            item.minimumThickness = 200
            splitView.addArrangedSubview(childView)

            // 첫 번째 자식에 holding priority 설정 (리사이즈 스냅 방지)
            if index == 0 {
                splitView.setHoldingPriority(.defaultHigh - 1, forSubviewAt: 0)
            }
        }

        // 초기 ratio 적용
        context.coordinator.pendingRatios[node.id] = node.ratio
        context.coordinator.registerSplitView(splitView, paneId: node.id)

        return splitView
    }

    // MARK: - Coordinator

    public final class Coordinator: NSObject, NSSplitViewDelegate {
        private let model: PaneTreeModel
        private var activePaneId: Binding<Int64?>
        var pendingRatios: [Int64: Double] = [:]
        private var splitViewPaneIds: [ObjectIdentifier: Int64] = [:]

        init(model: PaneTreeModel, activePaneId: Binding<Int64?>) {
            self.model = model
            self.activePaneId = activePaneId
        }

        func registerSplitView(_ splitView: NSSplitView, paneId: Int64) {
            splitViewPaneIds[ObjectIdentifier(splitView)] = paneId
        }

        // 최소 pane 크기 200pt 강제
        public func splitView(
            _ splitView: NSSplitView,
            constrainMinCoordinate proposedMinimumPosition: CGFloat,
            ofSubviewAt dividerIndex: Int
        ) -> CGFloat {
            // @MX:NOTE: [AUTO] 200pt 최소 크기 — SPEC AC-1.3 요구사항
            return max(proposedMinimumPosition, 200)
        }

        public func splitView(
            _ splitView: NSSplitView,
            constrainMaxCoordinate proposedMaximumPosition: CGFloat,
            ofSubviewAt dividerIndex: Int
        ) -> CGFloat {
            let total = splitView.isVertical ? splitView.bounds.width : splitView.bounds.height
            return min(proposedMaximumPosition, total - 200)
        }

        // 드래그 후 ratio 영속
        public func splitViewDidResizeSubviews(_ notification: Notification) {
            guard let splitView = notification.object as? NSSplitView,
                  let paneId = splitViewPaneIds[ObjectIdentifier(splitView)],
                  splitView.subviews.count == 2
            else { return }

            let total = splitView.isVertical ? splitView.bounds.width : splitView.bounds.height
            guard total > 0 else { return }

            let firstSize = splitView.isVertical
                ? splitView.subviews[0].bounds.width
                : splitView.subviews[0].bounds.height
            let ratio = Double(firstSize / total)

            Task { @MainActor in
                self.model.updateRatio(paneId, ratio: ratio)
            }
        }
    }
}

// MARK: - MoAISplitView

/// NSSplitView 서브클래스 — 커스텀 divider 색상 제공.
private final class MoAISplitView: NSSplitView {
    override var dividerColor: NSColor {
        NSColor.separatorColor
    }
}

// MARK: - LeafPlaceholderView

/// MS-3 이전 leaf pane 에 표시되는 임시 플레이스홀더 뷰.
// @MX:NOTE: [AUTO] MS-3 에서 TabBarView + SurfaceProtocol 구현 후 이 뷰를 교체.
struct LeafPlaceholderView: View {
    let paneId: Int64
    @Binding var activePaneId: Int64?

    private var isActive: Bool { activePaneId == paneId }

    var body: some View {
        ZStack {
            // 활성 pane 강조 배경
            RoundedRectangle(cornerRadius: 4)
                .fill(isActive
                    ? Color.accentColor.opacity(0.08)
                    : Color(NSColor.windowBackgroundColor))

            VStack(spacing: 8) {
                Image(systemName: "square.split.2x1")
                    .font(.system(size: 32))
                    .foregroundColor(.secondary)
                Text("Pane \(paneId)")
                    .font(.headline)
                    .foregroundColor(.primary)
                Text("MS-3 에서 Surface 연결 예정")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .contentShape(Rectangle())
        .onTapGesture {
            activePaneId = paneId
        }
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .strokeBorder(isActive ? Color.accentColor : Color.clear, lineWidth: 1)
        )
    }
}

// MARK: - PaneSplitContainerView (단축키 포함 래퍼)

/// 단축키(T-040)와 PaneSplitView 를 통합한 컨테이너 뷰.
// @MX:ANCHOR: [AUTO] 단축키 → PaneTreeModel 변이 → PaneSplitView 렌더링의 통합 진입점 (fan_in>=3)
// @MX:REASON: [AUTO] PaneContainer, 키보드 단축키 핸들러, 테스트 세 경로에서 참조
public struct PaneSplitContainerView: View {
    @Bindable var model: PaneTreeModel
    @State private var activePaneId: Int64?

    public init(model: PaneTreeModel) {
        self.model = model
    }

    public var body: some View {
        PaneSplitView(model: model, activePaneId: $activePaneId)
            .onAppear {
                // 초기 활성 pane 설정
                if activePaneId == nil {
                    activePaneId = model.rootId
                }
            }
            // T-040: Cmd+\ — 수평 분할 (좌우)
            .onKeyboardShortcut(.init("\\", modifiers: .command)) {
                guard let paneId = activePaneId else { return }
                let newId = model.splitActive(paneId, direction: .horizontal)
                if let newId { activePaneId = newId }
            }
            // T-040: Cmd+Shift+\ — 수직 분할 (상하)
            .onKeyboardShortcut(.init("\\", modifiers: [.command, .shift])) {
                guard let paneId = activePaneId else { return }
                let newId = model.splitActive(paneId, direction: .vertical)
                if let newId { activePaneId = newId }
            }
            // T-040: Cmd+Shift+W — 활성 pane 닫기
            .onKeyboardShortcut(.init("w", modifiers: [.command, .shift])) {
                guard let paneId = activePaneId else { return }
                let closed = model.closePane(paneId)
                if closed {
                    activePaneId = model.rootId
                }
            }
    }
}

// MARK: - View 확장 (onKeyboardShortcut 헬퍼)

private extension View {
    /// SwiftUI .commands 없이 View 레벨에서 키보드 단축키를 처리하는 헬퍼.
    // @MX:NOTE: [AUTO] NSSplitView 포커스 관리와 SwiftUI .keyboardShortcut 충돌 방지를 위해
    //            NSViewRepresentable 레이어 대신 SwiftUI overlay 에서 처리.
    func onKeyboardShortcut(_ shortcut: KeyboardShortcut, action: @escaping () -> Void) -> some View {
        self.background(
            Button("") { action() }
                .keyboardShortcut(shortcut)
                .frame(width: 0, height: 0)
                .opacity(0)
        )
    }
}
