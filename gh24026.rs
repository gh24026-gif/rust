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
