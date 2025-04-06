# Templates

This directory contains the templates that are used by the tool. These are
included in the executables you find in releases so unless you are tinkering
with your own templates you need not even look here. However if you are looking
to tinker with the templates keep reading.

## Internal templates

As mentioned previously these templates can be compiled in statically into the
executable. Wasteful? perhaps but greatly reduces the hassle of setting up the
tool and allows for older versions to remain functional after updates to this
directory. By default they are not compiled in and you need to enable the
```internal``` feature on compile time. You can force the use of the internal
templates via the
```--use-internal``` flag on runtime, and they will be used by default if the ```templates```
directory is not in the cwd.

## External templates

If the ```templates``` directory is present then the tool will attempt to load
the templates located there. It's looking for files with the following names
```homeTemplate```, ```cultureTemplate```, ```dynastyTemplate```, ```faithTemplate```, ```homeTemplate```, ```timelineTemplate```, ```titleTemplate```, ```houseTemplate```
ignoring extensions so you can theoretically provide, for example, Markdown
templates and have the utility create Markdown files. All of these utilize the
Jinja extension system. The template that contains the shared body is called
```base```. Thanks to all this you can, with very little work, change the output
format. Like want to develop a template set that looks like parchment? Feel free
to do so.

## Custom templates

Creation of custom templates is something I greatly encourage. I would love to
see template packs or other similar things. Just be sure you use the same Jinja
variables as I do and your templates should work fine.
