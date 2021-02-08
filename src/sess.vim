let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd ~/Projects/gaiasky-catgen/src
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
set shortmess=aoO
badd +348 main.rs
badd +457 lod.rs
badd +437 load.rs
badd +160 data.rs
badd +1 coord.rs
badd +1 util.rs
badd +0 parse.rs
badd +78 write.rs
badd +14 ~/Projects/gaiasky-catgen/Cargo.toml
argglobal
%argdel
$argadd main.rs
set stal=2
tabnew
tabnew
tabnew
tabnew
tabnew
tabnew
tabnew
tabrewind
edit main.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd t
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 348 - ((27 * winheight(0) + 15) / 31)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
348
normal! 050|
wincmd w
argglobal
enew
file -m-
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
wincmd w
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
tabnext
edit write.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd t
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 78 - ((16 * winheight(0) + 15) / 31)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
78
normal! 038|
wincmd w
argglobal
enew
file -m-
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
wincmd w
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
tabnext
edit lod.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd t
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 124 - ((15 * winheight(0) + 15) / 31)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
124
normal! 0
wincmd w
argglobal
enew
file -m-
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
lcd ~/Projects/gaiasky-catgen/src
wincmd w
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
tabnext
edit ~/Projects/gaiasky-catgen/src/load.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd t
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 437 - ((26 * winheight(0) + 15) / 31)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
437
normal! 058|
wincmd w
argglobal
enew
file ~/Projects/gaiasky-catgen/src/-m-
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
lcd ~/Projects/gaiasky-catgen/src
wincmd w
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
tabnext
edit ~/Projects/gaiasky-catgen/src/data.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd t
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 12 - ((11 * winheight(0) + 15) / 31)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
12
normal! 012|
wincmd w
argglobal
enew
file ~/Projects/gaiasky-catgen/src/-m-
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
lcd ~/Projects/gaiasky-catgen/src
wincmd w
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
tabnext
edit ~/Projects/gaiasky-catgen/src/coord.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd t
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 1 - ((0 * winheight(0) + 20) / 40)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
1
normal! 0
lcd ~/Projects/gaiasky-catgen/src
wincmd w
argglobal
enew
file ~/Projects/gaiasky-catgen/src/-m-
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
lcd ~/Projects/gaiasky-catgen/src
wincmd w
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
tabnext
edit ~/Projects/gaiasky-catgen/src/util.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd t
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 1 - ((0 * winheight(0) + 20) / 40)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
1
normal! 0
lcd ~/Projects/gaiasky-catgen/src
wincmd w
argglobal
enew
file ~/Projects/gaiasky-catgen/src/-m-
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
lcd ~/Projects/gaiasky-catgen/src
wincmd w
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
tabnext
edit ~/Projects/gaiasky-catgen/src/parse.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd t
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
argglobal
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 1 - ((0 * winheight(0) + 20) / 40)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
1
normal! 0
wincmd w
argglobal
enew
file ~/Projects/gaiasky-catgen/src/-m-
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
lcd ~/Projects/gaiasky-catgen/src
wincmd w
exe 'vert 1resize ' . ((&columns * 201 + 106) / 212)
exe 'vert 2resize ' . ((&columns * 10 + 106) / 212)
tabnext 2
set stal=1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0&& getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20 winminheight=1 winminwidth=1 shortmess=filnxtToOFA
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
