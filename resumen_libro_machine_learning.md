# Resumen: Practical Machine Learning with Rust

## Cap1: Ya lo se todo(...ponele)

## Cap2: Supervised Learning

### Que es machine learning?

Es la ciencia de hacer que las computadoras actuen sin necesariamente estar
programadas especificamente. Esto se puede llevar a cabo implementando unos
algoritmos que tienen la particularidad de detectar patrones en los datos

#### Supervised Learning

Cuando le pasamos al sistema la entrada y la salida deseada y lo que queremos
que el sistema "capture" la relacion que hay entre ellos. A su vez podemos
dividir en dos categorias:

 - Supervised Task: Donde la variable de target es continua. Por ejemplo el
   precio de un producto puede tener cualquier valor

 - Problema de clasificacion: En este caso los labels son discretos y finitos.
   Por ejemplo podemos categorizar los emails que llegan como spam o no spam


#### Unsupervised Learning

Aqui los labels o las clases target no son dadas. Por ello la meta de esta
tecnica es encontrar una particion natural de patrones en los datos. Una de las
aplicaciones que puede tener es cuando los datos sobre una salida deseada no son
conocidos, como cuando creamos una nueva clase de producto en un mercado en el
que nunca se ha vendido


#### Reinforcement Learning

Aqui agentes que son hechos a mano son puestos en marcha en un ambiente, en el
cual ellos deberan tomar acciones basadas sobre un sistema de recompensa. Este
paradigma ha sido utilizado en Robotica por ejemplo para hacer navegacion.


### Codigo especifico para los Datasets

Vamos a crear un nuevo programa en Rust para realizar una regresion, llamado
`regression`

En el libro verifica los datos primero con unas magias de bash, y en el caso de
su dataset no tiene headers y tiene un numero fijo de columnas(me parece que en
el que baje yo si tengo headers)

 - Necesitamos "parsear" el csv para extraer los datos, para ello vamos a
   utilizar una libreria

 - Vamos a considerar al valor medio de los precios de las casas


