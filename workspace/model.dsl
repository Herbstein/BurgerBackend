workspace "Burger Backend" "Burger Backend 1.0™" {

    model {
        user = person "User" "Burger Lover"

        burger = softwareSystem "Burger Backend" {
            webapp = container "Backend" "Optimized for template rendering" "Rust"
            database = container "Database" "Contained in-memory on Backend"
            front = container "Frontend" "Thin HTML + CSS Client"
            blob = container "Blob Storage" "File system next to source code"
        }

        email = softwareSystem  "Email System"
        
        logs = softwareSystem "Log Management"

        burger -> email "Send verification and recovery emails"
        burger -> logs "Forward logs to log management"
        
        email -> user "Send verification and notficiations"
        user -> front "Finds restaurants and posts reviews"
        
        front -> webapp "Forwards interactions via HTTP"
        
        webapp -> front "Sends pre-rendered HTML"
        webapp -> database "Reads from and writes to"
        webapp -> blob "Store uploaded images"
    }

    views {
        container burger {
            include *
            autolayout
        }

        systemContext burger {
            include *
            autolayout
        }
    }

}
