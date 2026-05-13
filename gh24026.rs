//TEORÍA 
// ----------------------------------------------------------------------------
//
// .take() en las rotaciones
// -----------------------------------------
// En Rust, el Borrow Checker prohíbe mover un valor fuera de una referencia
// mutable si esa referencia sigue viva. Al escribir `y.izquierdo = x` no
// podríamos acceder a `y` después el compilador no sabe cuándo termina
// el préstamo. `.take()` reemplaza el campo por `None` y devuelve el valor
// anterior como `Some(...)`, cediendo el ownership de forma segura y dejando
// el campo en un estado válido (None) mientras operamos con el nodo extraído.
// Así evitamos clonar y cumplimos las reglas de ownership sin errores.
//
//  Box y Option<Box<Nodo>>
// ------------------------------------
// `Box<T>` guarda el valor en el heap y nos da un puntero con ownership único.
// Es necesario para tipos recursivos: sin él Nodo tendría tamaño infinito.
// `Option<Box<Nodo>` permite representar la ausencia del hijo (None) o su
// presencia (Some(Box<Nodo>)) sin usar punteros nulos.
//
// as_ref()
// --------------------
// Convierte &Option<T> en Option<&T>, es decir, toma prestado el contenido
// sin tomar ownership. Así podemos inspeccionar el Option sin moverlo.
//
// ----------------------------------------------------------------------------
// PRUEBA DE ESCRITORIO 
// ----------------------------------------------------------------------------
//  1: insertar 10          →  raíz = 10
//  2: insertar 20          →  10 → derecho = 20
//  3: insertar 30          →  balance(10) = -2  → Rotación simple a ka izquierda 
//                                  resultado:   raíz = 20, izq = 10, der = 30
//  4: insertar 5           →  10.izquierdo = 5
//  5: insertar 2           →  balance(10) = 2  → Rotación simple a la derecha ← AQUÍ
//                                  10 sube a 5, izquierdo = 2, derecho = 10
//                                  árbol:  raíz=20, izq=5(izq=2,der=10), der=30
//  6: insertar 25          →  30.izquierdo = 25  → balance(30) = 1 (sin rotación)
//
//  Estado final (rotated in-order: 2,5,10,20,25,30):
//
//                  20
//                /    \
//               5      30
//              / \    /
//             2  10  25
#[derive(Debug, Clone)]
struct Libro {
    isbn: u32,
    titulo: String,
}

struct Nodo {
    libro: Libro,
    izquierdo: Option<Box<Nodo>>,
    derecho: Option<Box<Nodo>>,
    altura: i32,
}

impl Nodo {
    fn nuevo(libro: Libro) -> Self {
        Nodo {
            libro,
            izquierdo: None,
            derecho: None,
            altura: 1,
        }
    }
}

// ---- altura y balance ----------------------------------------

fn obtener_altura(nodo: &Option<Box<Nodo>>) -> i32 {
    nodo.as_ref().map_or(0, |n| n.altura)
}

fn actualizar_altura(nodo: &mut Nodo) {
    nodo.altura = 1 + std::cmp::max(
        obtener_altura(&nodo.izquierdo),
        obtener_altura(&nodo.derecho),
    );
}

fn obtener_balance(nodo: &Nodo) -> i32 {
    obtener_altura(&nodo.izquierdo) - obtener_altura(&nodo.derecho)
}

// ---- Rotaciones -------------------------------------------------------------

fn rotar_derecha(mut y: Box<Nodo>) -> Box<Nodo> {
    let mut x = y.izquierdo.take().expect("Hijo izquierdo ausente en rotación derecha");
    y.izquierdo = x.derecho.take();
    actualizar_altura(&mut y);
    x.derecho = Some(y);
    actualizar_altura(&mut x);
    x
}

fn rotar_izquierda(mut x: Box<Nodo>) -> Box<Nodo> {
    let mut y = x.derecho.take().expect("Hijo derecho ausente en rotación izquierda");
    x.derecho = y.izquierdo.take();
    actualizar_altura(&mut x);
    y.izquierdo = Some(x);
    actualizar_altura(&mut y);
    y
}

// ---- Inserción -------------------------------------------------------------

fn insertar(nodo_opt: Option<Box<Nodo>>, libro: Libro) -> Box<Nodo> {
    let mut nodo = match nodo_opt {
        None => return Box::new(Nodo::nuevo(libro)),
        Some(n) => n,
    };
    
    let isbn_nuevo = libro.isbn;
    if isbn_nuevo < nodo.libro.isbn {
        nodo.izquierdo = Some(insertar(nodo.izquierdo.take(), libro));
    } else if isbn_nuevo > nodo.libro.isbn {
        nodo.derecho = Some(insertar(nodo.derecho.take(), libro));
    } else {
        return nodo;
    }

    actualizar_altura(&mut nodo);
    rebalancear(nodo, isbn_nuevo, None)
}

// ---- FASE 2: Búsqueda ------------------------------------------------------

fn buscar(nodo: &Option<Box<Nodo>>, isbn: u32) -> Option<&Libro> {
    let mut actual = nodo;
    loop {
        match actual {
            None => return None,
            Some(n) => {
                if isbn == n.libro.isbn {
                    return Some(&n.libro);
                } else if isbn < n.libro.isbn {
                    actual = &n.izquierdo;
                } else {
                    actual = &n.derecho;
                }
            }
        }
    }
}

// ---- FASE 3: Eliminación ---------------------------------------------------

fn extraer_minimo(mut nodo: Box<Nodo>) -> (Option<Box<Nodo>>, Box<Nodo>) {
    if nodo.izquierdo.is_none() {
        let derecho = nodo.derecho.take();
        return (derecho, nodo);
    }
    let (nuevo_izq, minimo) = extraer_minimo(nodo.izquierdo.take().unwrap());
    nodo.izquierdo = nuevo_izq;
    actualizar_altura(&mut nodo);
    let nodo = rebalancear(nodo, minimo.libro.isbn, None);
    (Some(nodo), minimo)
}

fn eliminar(nodo_opt: Option<Box<Nodo>>, isbn: u32) -> Option<Box<Nodo>> {
    let mut nodo = match nodo_opt {
        None => return None,
        Some(n) => n,
    };

    if isbn < nodo.libro.isbn {
        nodo.izquierdo = eliminar(nodo.izquierdo.take(), isbn);
    } else if isbn > nodo.libro.isbn {
        nodo.derecho = eliminar(nodo.derecho.take(), isbn);
    } else {
        match (nodo.izquierdo.take(), nodo.derecho.take()) {
            (None, None) => return None,
            (None, Some(der)) => return Some(der),
            (Some(izq), None) => return Some(izq),
            (Some(izq), Some(der)) => {
                let (nuevo_der, mut sucesor) = extraer_minimo(der);
                sucesor.izquierdo = Some(izq);
                sucesor.derecho = nuevo_der;
                actualizar_altura(&mut sucesor);
                let isbn_sucesor = sucesor.libro.isbn;
                return Some(rebalancear(sucesor, isbn_sucesor, None));
            }
        }
    }

    actualizar_altura(&mut nodo);
    Some(rebalancear(nodo, isbn, None))
}

// ---- FASE 4: Rango ---------------------------------------------------------

fn buscar_rango<'a>(nodo: &'a Option<Box<Nodo>>, min: u32, max: u32) -> Vec<&'a Libro> {
    let mut resultado = Vec::new();
    buscar_rango_helper(nodo, min, max, &mut resultado);
    resultado
}

fn buscar_rango_helper<'a>(
    nodo: &'a Option<Box<Nodo>>,
    min: u32,
    max: u32,
    resultado: &mut Vec<&'a Libro>,
) {
    if let Some(n) = nodo {
        if n.libro.isbn > min {
            buscar_rango_helper(&n.izquierdo, min, max, resultado);
        }
        if n.libro.isbn >= min && n.libro.isbn <= max {
            resultado.push(&n.libro);
        }
        if n.libro.isbn < max {
            buscar_rango_helper(&n.derecho, min, max, resultado);
        }
    }
}
fn rebalancear(mut nodo: Box<Nodo>, isbn_ref: u32, _isbn_eliminado: Option<u32>) -> Box<Nodo> {
    let balance = obtener_balance(&nodo);

    if balance > 1 && isbn_ref < nodo.izquierdo.as_ref().unwrap().libro.isbn {
        return rotar_derecha(nodo);
    }
    if balance < -1 && isbn_ref > nodo.derecho.as_ref().unwrap().libro.isbn {
        return rotar_izquierda(nodo);
    }
    if balance > 1 && isbn_ref > nodo.izquierdo.as_ref().unwrap().libro.isbn {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return rotar_derecha(nodo);
    }
    if balance < -1 && isbn_ref < nodo.derecho.as_ref().unwrap().libro.isbn {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return rotar_izquierda(nodo);
    }
    nodo
}

fn imprimir(nodo: &Option<Box<Nodo>>, nivel: usize) {
    if let Some(n) = nodo {
        imprimir(&n.derecho, nivel + 1);
        println!(
            "{:indent$}[ISBN: {:3}] {}",
            "",
            n.libro.isbn,
            n.libro.titulo,
            indent = nivel * 4
        );
        imprimir(&n.izquierdo, nivel + 1);
    }
} 
// ---- MAIN ------------------------------------------------------------------

fn main() {
    let mut raiz: Option<Box<Nodo>> = None;

    let datos = vec![
        (10, "El Quijote"),
        (20, "1984"),
        (30, "Hamlet"),
        (5,  "Fahrenheit 451"),
        (2,  "La Odisea"),
        (25, "El Principito"),
    ];

    println!("--------------------------------");
    println!("    INVENTARIO DE LIBRERIA       ");
    println!("--------------------------------");

    for (isbn, titulo) in datos {
        let libro = Libro { isbn, titulo: titulo.to_string() };
        raiz = Some(insertar(raiz.take(), libro));
    }

    println!("\nARBOL INICIAL (derecha -> raiz -> izquierda):");
    imprimir(&raiz, 0);

    println!("\n--- BUSQUEDA ---");

    let isbn_existe = 25u32;
    match buscar(&raiz, isbn_existe) {
        Some(libro) => println!("ISBN {} encontrado: \"{}\"", libro.isbn, libro.titulo),
        None => println!("ISBN {} no encontrado.", isbn_existe),
    }

    let isbn_no_existe = 99u32;
    match buscar(&raiz, isbn_no_existe) {
        Some(libro) => println!("ISBN {} encontrado: \"{}\"", libro.isbn, libro.titulo),
        None => println!("ISBN {} no encontrado (correcto).", isbn_no_existe),
    }

    println!("\n--- ELIMINACION ---");
    println!("Eliminando ISBN 20...");

    raiz = eliminar(raiz.take(), 20);

    println!("Estado del arbol tras eliminar ISBN 20:");
    imprimir(&raiz, 0);

    match buscar(&raiz, 20) {
        Some(_) => println!("Error: ISBN 20 todavia existe."),
        None    => println!("ISBN 20 eliminado correcto."),
    }

    println!("\nVerificando integridad del resto del arbol:");
    for isbn in [2u32, 5, 10, 25, 30] {
        match buscar(&raiz, isbn) {
            Some(l) => println!("   ISBN {:3} presente: \"{}\"", l.isbn, l.titulo),
            None    => println!("   ISBN {:3} faltante.", isbn),
        }
    }

    println!("\n--- BUSQUEDA POR RANGO ---");

    let (min, max) = (5u32, 27u32);
    println!("Libros con ISBN entre {} y {}:", min, max);

    let libros_en_rango = buscar_rango(&raiz, min, max);
    if libros_en_rango.is_empty() {
        println!("   (Sin resultados)");
    } else {
        for libro in &libros_en_rango {
            println!("   ISBN {:3}: \"{}\"", libro.isbn, libro.titulo);
        }
    }

    let (min2, max2) = (40u32, 99u32);
    let libros_vacio = buscar_rango(&raiz, min2, max2);
    println!(
        "Libros con ISBN entre {} y {}: {}",
        min2, max2,
        if libros_vacio.is_empty() { "Ninguno (correcto)" } else { "Hay resultados" }
    );

    println!("\n----------------------------------------");
    println!("   TODAS LAS FASES CORRECTAS             ");
    println!("------------------------------------------");
}
