import math
from vapoursynth import core
import vapoursynth as vs

"""
ChangeFPS
Overlay
InterFrame
"""

def ChangeFPS(clip, fpsnum, fpsden=1):
    if not isinstance(clip, vs.VideoNode):
        raise vs.Error('ChangeFPS: This is not a clip')

    factor = (fpsnum / fpsden) * (clip.fps_den / clip.fps_num)

    def frame_adjuster(n):
        real_n = math.floor(n / factor)
        one_frame_clip = clip[real_n] * (len(clip) + 100)
        return one_frame_clip

    attribute_clip = clip.std.BlankClip(length=math.floor(len(clip) * factor), fpsnum=fpsnum, fpsden=fpsden)
    return attribute_clip.std.FrameEval(eval=frame_adjuster)

# Available blend modes:
#  normal
#  addition
#  average
#  burn
#  darken
#  difference
#  divide
#  dodge
#  exclusion
#  extremity
#  freeze
#  glow
#  grainextract
#  grainmerge
#  hardlight
#  hardmix
#  heat
#  lighten
#  linearlight
#  multiply
#  negation
#  overlay
#  phoenix
#  pinlight
#  reflect
#  screen
#  softlight
#  subtract
#  vividlight
def Overlay(base, overlay, x=0, y=0, mask=None, opacity=1.0, mode='normal', planes=None, mask_first_plane=True):
    if not (isinstance(base, vs.VideoNode) and isinstance(overlay, vs.VideoNode)):
        raise vs.Error('Overlay: This is not a clip')

    if mask is not None:
        if not isinstance(mask, vs.VideoNode):
            raise vs.Error("Overlay: 'mask' is not a clip")

        if mask.width != overlay.width or mask.height != overlay.height or mask.format.bits_per_sample != overlay.format.bits_per_sample:
            raise vs.Error("Overlay: 'mask' must have the same dimensions and bit depth as 'overlay'")

    if base.format.sample_type == vs.INTEGER:
        neutral = 1 << (base.format.bits_per_sample - 1)
        peak = (1 << base.format.bits_per_sample) - 1
        factor = 1 << base.format.bits_per_sample
    else:
        neutral = 0.5
        peak = factor = 1.0

    if planes is None:
        planes = list(range(base.format.num_planes))
    elif isinstance(planes, int):
        planes = [planes]

    if base.format.subsampling_w > 0 or base.format.subsampling_h > 0:
        base_orig = base
        base = base.resize.Point(format=base.format.replace(subsampling_w=0, subsampling_h=0))
    else:
        base_orig = None

    if overlay.format.id != base.format.id:
        overlay = overlay.resize.Point(format=base.format)

    if mask is None:
        mask = overlay.std.BlankClip(format=overlay.format.replace(color_family=vs.GRAY, subsampling_w=0, subsampling_h=0), color=[peak])
    elif mask.format.id != overlay.format.id and mask.format.color_family != vs.GRAY:
        mask = mask.resize.Point(format=overlay.format, range_s='full')

    opacity = min(max(opacity, 0.0), 1.0)
    mode = mode.lower()

    # Calculate padding sizes
    l, r = x, base.width - overlay.width - x
    t, b = y, base.height - overlay.height - y

    # Split into crop and padding values
    cl, pl = min(l, 0) * -1, max(l, 0)
    cr, pr = min(r, 0) * -1, max(r, 0)
    ct, pt = min(t, 0) * -1, max(t, 0)
    cb, pb = min(b, 0) * -1, max(b, 0)

    # Crop and padding
    overlay = overlay.std.Crop(left=cl, right=cr, top=ct, bottom=cb)
    overlay = overlay.std.AddBorders(left=pl, right=pr, top=pt, bottom=pb)
    mask = mask.std.Crop(left=cl, right=cr, top=ct, bottom=cb)
    mask = mask.std.AddBorders(left=pl, right=pr, top=pt, bottom=pb, color=[0] * mask.format.num_planes)

    if opacity < 1:
        mask = mask.std.Expr(expr=[f'x {opacity} *'])

    if mode == 'normal':
        pass
    elif mode == 'addition':
        expr = f'x y +'
    elif mode == 'average':
        expr = f'x y + 2 /'
    elif mode == 'burn':
        expr = f'x 0 <= x {peak} {peak} y - {factor} * x / - ?'
    elif mode == 'darken':
        expr = f'x y min'
    elif mode == 'difference':
        expr = f'x y - abs'
    elif mode == 'divide':
        expr = f'y 0 <= {peak} {peak} x * y / ?'
    elif mode == 'dodge':
        expr = f'x {peak} >= x y {factor} * {peak} x - / ?'
    elif mode == 'exclusion':
        expr = f'x y + 2 x * y * {peak} / -'
    elif mode == 'extremity':
        expr = f'{peak} x - y - abs'
    elif mode == 'freeze':
        expr = f'y 0 <= 0 {peak} {peak} x - dup * y / {peak} min - ?'
    elif mode == 'glow':
        expr = f'x {peak} >= x y y * {peak} x - / ?'
    elif mode == 'grainextract':
        expr = f'x y - {neutral} +'
    elif mode == 'grainmerge':
        expr = f'x y + {neutral} -'
    elif mode == 'hardlight':
        expr = f'y {neutral} < 2 y x * {peak} / * {peak} 2 {peak} y - {peak} x - * {peak} / * - ?'
    elif mode == 'hardmix':
        expr = f'x {peak} y - < 0 {peak} ?'
    elif mode == 'heat':
        expr = f'x 0 <= 0 {peak} {peak} y - dup * x / {peak} min - ?'
    elif mode == 'lighten':
        expr = f'x y max'
    elif mode == 'linearlight':
        expr = f'y {neutral} < y 2 x * + {peak} - y 2 x {neutral} - * + ?'
    elif mode == 'multiply':
        expr = f'x y * {peak} /'
    elif mode == 'negation':
        expr = f'{peak} {peak} x - y - abs -'
    elif mode == 'overlay':
        expr = f'x {neutral} < 2 x y * {peak} / * {peak} 2 {peak} x - {peak} y - * {peak} / * - ?'
    elif mode == 'phoenix':
        expr = f'x y min x y max - {peak} +'
    elif mode == 'pinlight':
        expr = f'y {neutral} < x 2 y * min x 2 y {neutral} - * max ?'
    elif mode == 'reflect':
        expr = f'y {peak} >= y x x * {peak} y - / ?'
    elif mode =='screen':
        expr = f'{peak} {peak} x - {peak} y - * {peak} / -'
    elif mode == 'softlight':
        expr = f'x {neutral} > y {peak} y - x {neutral} - * {neutral} / 0.5 y {neutral} - abs {peak} / - * + y y {neutral} x - {neutral} / * 0.5 y {neutral} - abs {peak} / - * - ?'
    elif mode == 'subtract':
        expr = f'x y -'
    elif mode == 'vividlight':
        expr = f'x {neutral} < x 0 <= 2 x * {peak} {peak} y - {factor} * 2 x * / - ? 2 x {neutral} - * {peak} >= 2 x {neutral} - * y {factor} * {peak} 2 x {neutral} - * - / ? ?'
    else:
        raise vs.Error("Overlay: invalid 'mode' specified")

    if mode != 'normal':
        overlay = core.std.Expr([overlay, base], expr=[expr if i in planes else '' for i in range(base.format.num_planes)])
    last = core.std.MaskedMerge(base, overlay, mask, planes=planes, first_plane=mask_first_plane)

    # Return padded clip
    if base_orig is not None:
        last = core.resize.Point(last, format=base_orig.format)
    return last

"""
Adjusted InterFrame from havsfunc,support 10bit with new svp
Source: https://github.com/xyx98/my-vapoursynth-script/blob/master/xvs.py
"""
def InterFrame(Input, Preset='Medium', Tuning='Film', NewNum=None, NewDen=1, GPU=True, gpuid=0, OverrideAlgo=None, OverrideArea=None,
               FrameDouble=False, OverrideBlockSize='auto'):

    if not isinstance(Input, vs.VideoNode):
        raise TypeError('InterFrame: This is not a video')

    sw=Input.format.subsampling_w
    sh=Input.format.subsampling_h
    depth= Input.format.bits_per_sample
    if  not sw ==1 and not sh==1 and depth not in [8,10]:
        raise TypeError('InterFrame: input must be yuv420p8 or yuv420p10')
    oInput=Input
    # Input = core.resize.Point(Input, vs.YUV420P8) # what else would it expect anyways? (im sorry i didnt mean to sound cocky please pr please help please help)
    Input = core.fmtc.bitdepth(Input,bits=8)
    # Validate inputs
    Preset = Preset.lower()
    Tuning = Tuning.lower()
    if Preset not in ['medium', 'fast', 'faster', 'fastest']:
        raise ValueError("InterFrame: '{Preset}' is not a valid preset".format(Preset=Preset))
    if Tuning not in ['film', 'smooth', 'animation', 'weak']:
        raise ValueError("InterFrame: '{Tuning}' is not a valid tuning".format(Tuning=Tuning))
    
    def InterFrameProcess(clip,oclip):
        # Create SuperString
        if Preset in ['fast', 'faster', 'fastest']:
            SuperString = '{pel:1,'
        else:
            SuperString = '{'
        
        SuperString += 'gpu:1}' if GPU else 'gpu:0}'
        
        # Create VectorsString
        if OverrideBlockSize not in [None, '', 'auto']:
            if 'x' in OverrideBlockSize: # contains a width, and height
                width, height = OverrideBlockSize.split('x')
                VectorsString = '{block:{w:' + width + ','
                if height != '':
                    VectorsString += 'h:' + height + ','
            else: # only contains width
                VectorsString = '{block:{w:' + OverrideBlockSize + ','
        else:
            if Tuning == 'animation' or Preset == 'fastest':
                VectorsString = '{block:{w:32,'
            elif Preset in ['fast', 'faster'] or not GPU:
                VectorsString = '{block:{w:16,'
            else:
                VectorsString = '{block:{w:8,'
        
        if Tuning == 'animation' or Preset == 'fastest':
            VectorsString += 'overlap:0'
        elif Preset == 'faster' and GPU:
            VectorsString += 'overlap:1'
        else:
            VectorsString += 'overlap:2'
        
        if Tuning == 'animation':
            VectorsString += '},main:{search:{coarse:{type:2,'
        elif Preset == 'faster':
            VectorsString += '},main:{search:{coarse:{'
        else:
            VectorsString += '},main:{search:{distance:0,coarse:{'
        
        if Tuning == 'animation':
            VectorsString += 'distance:-6,satd:false},distance:0,'
        elif Tuning == 'weak':
            VectorsString += 'distance:-1,trymany:true,'
        else:
            VectorsString += 'distance:-10,'
        
        if Tuning == 'animation' or Preset in ['faster', 'fastest']:
            VectorsString += 'bad:{sad:2000}}}}}'
        elif Tuning == 'weak':
            VectorsString += 'bad:{sad:2000}}}},refine:[{thsad:250,search:{distance:-1,satd:true}}]}'
        else:
            VectorsString += 'bad:{sad:2000}}}},refine:[{thsad:250}]}'
        
        # Create SmoothString
        if NewNum is not None:
            SmoothString = '{rate:{num:' + repr(NewNum) + ',den:' + repr(NewDen) + ',abs:true},'
        elif clip.fps_num / clip.fps_den in [15, 25, 30] or FrameDouble:
            SmoothString = '{rate:{num:2,den:1,abs:false},'
        else:
            SmoothString = '{rate:{num:60000,den:1001,abs:true},'
        if GPU:
            SmoothString+= 'gpuid:'+repr(gpuid)+','
        if OverrideAlgo is not None:
            SmoothString += 'algo:' + repr(OverrideAlgo) + ',mask:{cover:80,'
        elif Tuning == 'animation':
            SmoothString += 'algo:2,mask:{'
        elif Tuning == 'smooth':
            SmoothString += 'algo:23,mask:{'
        else:
            SmoothString += 'algo:13,mask:{cover:80,'
        
        if OverrideArea is not None:
            SmoothString += 'area:{OverrideArea}'.format(OverrideArea=OverrideArea)
        elif Tuning == 'smooth':
            SmoothString += 'area:150'
        else:
            SmoothString += 'area:0'
        
        if Tuning == 'weak':
            SmoothString += ',area_sharp:1.2},scene:{blend:true,mode:0,limits:{blocks:50}}}'
        else:
            SmoothString += ',area_sharp:1.2},scene:{blend:false,mode:0}}'
        
        # Make interpolation vector clip
        Super = core.svp1.Super(clip, SuperString)
        Vectors = core.svp1.Analyse(Super['clip'], Super['data'], clip, VectorsString)
        
        # Put it together
        return core.svp2.SmoothFps(oclip, Super['clip'], Super['data'], Vectors['clip'], Vectors['data'], SmoothString)
    

    return InterFrameProcess(Input,oInput)
