import QtQuick
import org.kde.plasma.plasmoid

WallpaperItem {
    id: root

    property string shaderName: root.configuration.ShaderName || ""
    property string paramsJson: root.configuration.Params || ""

    property real iTime: 0
    property real iTimeDelta: 0.016
    property int iFrame: 0
    property real startTime: 0

    property vector4d iResolution: Qt.vector4d(root.width, root.height, 1, 0)
    property vector4d iP0: Qt.vector4d(0, 0, 0, 1)
    property vector4d iP1: Qt.vector4d(0, 0, 0, 1)
    property vector4d iP2: Qt.vector4d(0, 0, 0, 1)
    property vector4d iP3: Qt.vector4d(0, 0, 0, 1)
    property vector4d iP4: Qt.vector4d(0, 0, 0, 1)
    property vector4d iP5: Qt.vector4d(0, 0, 0, 1)
    property vector4d iP6: Qt.vector4d(0, 0, 0, 1)
    property vector4d iP7: Qt.vector4d(0, 0, 0, 1)

    function loadParams() {
        if (!paramsJson) return;
        try {
            var arr = JSON.parse(paramsJson);
            for (var i = 0; i < 8 && i < arr.length; i++) {
                var v = arr[i];
                root["iP" + i] = Qt.vector4d(v[0], v[1], v[2], v[3]);
            }
        } catch (e) {}
    }

    onParamsJsonChanged: loadParams()
    onShaderNameChanged: {
        if (shaderName) {
            effect.fragmentShader = Qt.resolvedUrl("../shaders/" + shaderName + ".frag.qsb");
            startTime = Date.now() / 1000.0;
            iFrame = 0;
            timer.running = true;
        } else {
            timer.running = false;
        }
    }

    Component.onCompleted: {
        if (shaderName) {
            loadParams();
            effect.fragmentShader = Qt.resolvedUrl("../shaders/" + shaderName + ".frag.qsb");
            startTime = Date.now() / 1000.0;
            timer.running = true;
        }
    }

    Rectangle {
        anchors.fill: parent
        color: "black"
        visible: !shaderName
    }

    ShaderEffect {
        id: effect
        anchors.fill: parent
        visible: !!shaderName

        property alias iTime: root.iTime
        property alias iTimeDelta: root.iTimeDelta
        property alias iFrame: root.iFrame
        property real _pad: 0
        property alias iResolution: root.iResolution
        property alias iP0: root.iP0
        property alias iP1: root.iP1
        property alias iP2: root.iP2
        property alias iP3: root.iP3
        property alias iP4: root.iP4
        property alias iP5: root.iP5
        property alias iP6: root.iP6
        property alias iP7: root.iP7
    }

    Timer {
        id: timer
        interval: 16
        repeat: true
        running: false
        onTriggered: {
            var now = Date.now() / 1000.0;
            var prev = root.iTime;
            root.iTime = now - root.startTime;
            root.iTimeDelta = root.iTime - prev;
            root.iFrame++;
        }
    }
}
