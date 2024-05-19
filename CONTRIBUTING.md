# Welcome to the guide for contributors

Thank you for your interest in this project.
While I have so far worked entirely alone on this project it would be cool to get some help from time to time.
If you are capable of helping any contribution is entirely welcome.

## What needs to be done

Generally speaking this project has two main areas where you can find things to do or things that need to be done.

### HTML templates

If you have some experience with [Jinja](https://jinja.palletsprojects.com/en/3.1.x/)(or more specifically [minijinja](https://docs.rs/minijinja/latest/minijinja/)) templates or frontend development in general you can always try and improve the HTML [templates](./templates/).
I am not really a frontend guy, I'm certain there are things that could be improved in the design of the pages and the css styling.
If you see a way to improve the templates go ahead, create a fork and then do a pull request.
Alternatively if you want to create an entirely new set of templates with a different theme then also feel free to do so!
I am entirely open to creating a toggleable output theme switch within the program itself that would allow the user to choose between different themes.

### Rust sourcecode

This is the main area where help is most needed I would say.
Overall within the Rust source code you can help by: squashing bugs, optimising what I have written, writing documentation, improving accuracy with the in game state and adding new features.
Within the source code itself I mark areas that need special attention by adding comments that contain specific markers like ```TODO```, ```FIXME``` and ```MAYBE```.
If you work in VsCode I would advise you get [this](https://marketplace.visualstudio.com/items?itemName=Gruntfuggly.todo-tree) extension to mark them.

## Code guidelines

### HTML

There aren't many rules regarding HTML templates code.
Just make sure the HTML is properly indented to improve readability.

### Rust

Here there are a few more rules I would like you to follow.

1. No lint warnings - do not commit code that has any warnings
2. Each entity within the code must have it's comment documenting what it does - that goes for structs, traits and functions
3. Try to write optimal code - bit of a blanket statement, too vague to be actionable but still worth pointing out
