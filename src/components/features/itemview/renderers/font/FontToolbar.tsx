import { Component } from 'solid-js';
import { Type, AlignJustify, MoveVertical } from 'lucide-solid';
import { Popover } from '../../../../ui/Popover';
import { ColorPicker } from '../../../../ui/ColorPicker';
import { Slider } from '../../../../ui/Slider';
import { Button } from '../../../../ui/Button';
import { useItemViewContext, FontSettings } from '../../ItemViewContext';
import { Tooltip } from '../../../../ui/Tooltip';
import './font-view.css';

export const FontToolbar: Component = () => {
    const { fontSettings, setFontSettings } = useItemViewContext();

    const updateSetting = (key: keyof FontSettings, value: FontSettings[keyof FontSettings]) => {
        setFontSettings(prev => ({ ...prev, [key]: value }));
    };

    return (
        <>
            <div class="toolbar-group">
                <Tooltip position="bottom" content="Font Size">
                    <div class="font-control-group">
                        <Type size={14} class="font-icon" />
                        <div style={{ width: '80px' }}>
                            <Slider
                                minimumValue={8}
                                maximumValue={200}
                                showTicks={false}
                                value={fontSettings().fontSize}
                                onValueChange={newValue => updateSetting('fontSize', newValue)}
                            />
                        </div>
                        <span class="font-control-value">
                            {Math.round(fontSettings().fontSize)}
                        </span>
                    </div>
                </Tooltip>

                <div class="toolbar-separator" />

                <Tooltip position="bottom" content="Line Height">
                    <div class="font-control-group">
                        <MoveVertical size={14} class="font-icon" />
                        <div style={{ width: '80px' }}>
                            <Slider
                                minimumValue={0.5}
                                maximumValue={3}
                                stepValue={0.1}
                                value={fontSettings().lineHeight}
                                onValueChange={newValue => updateSetting('lineHeight', newValue)}
                                showTicks={false}
                            />
                        </div>
                    </div>
                </Tooltip>

                <Tooltip position="bottom" content="Letter Spacing">
                    <div class="font-control-group">
                        <AlignJustify
                            size={14}
                            class="font-icon"
                            style={{ transform: 'rotate(90deg)' }}
                        />
                        <div style={{ width: '80px' }}>
                            <Slider
                                minimumValue={-5}
                                maximumValue={20}
                                showTicks={false}
                                value={fontSettings().letterSpacing}
                                onValueChange={newValue => updateSetting('letterSpacing', newValue)}
                            />
                        </div>
                    </div>
                </Tooltip>
            </div>

            <div class="toolbar-group">
                <Popover
                    trigger={
                        <Button variant="ghost" class="font-color-btn">
                            <div
                                class="font-color-preview"
                                style={{ background: fontSettings().color }}
                            />
                            Text Color
                        </Button>
                    }
                >
                    <div class="font-color-popover">
                        <ColorPicker
                            color={fontSettings().color}
                            onChange={c => updateSetting('color', c)}
                        />
                    </div>
                </Popover>

                <Popover
                    trigger={
                        <Button variant="ghost" class="font-color-btn">
                            <div
                                class="font-color-preview"
                                style={{
                                    background: fontSettings().backgroundColor,
                                    'border-radius': '2px'
                                }}
                            />
                            Bg Color
                        </Button>
                    }
                >
                    <div class="font-color-popover">
                        <ColorPicker
                            color={fontSettings().backgroundColor}
                            onChange={c => updateSetting('backgroundColor', c)}
                            allowNoColor
                        />
                    </div>
                </Popover>
            </div>
        </>
    );
};
