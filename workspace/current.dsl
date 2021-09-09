workspace "Burger Backend" "" {

    model {
        user = person "User" "Burger Lover"
        burger = softwareSystem "Burger Backend" {
            webapp = container "Backend" "Optimized for template rendering"
            database = container "Database" "Contained in-memory on Backend"
            front = container "Frontend" "Thin HTML + CSS Client"
        }

        user -> front "Finds restaurants and posts reviews"
        front -> webapp "Forwards interactions via HTTP"
        webapp -> front "Sends pre-rendered HTML"
        webapp -> database "Reads from and writes to"
    }

    views {
        container burger {
            include *
            autoLayout lr
        }

        systemContext burger {
            include *
            autoLayout lr
        }
    }

}
