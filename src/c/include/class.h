#ifndef CLASS_H
#define CLASS_H

#include <setjmp.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define try                                                                    \
  jmp_buf ebuf;                                                                \
  int c = setjmp(ebuf);                                                        \
  if (c == 0) {

#define catch                                                                  \
  }                                                                            \
  else

#define throw(x) longjmp(ebuf, x)

#define NEW(ty, ...)                                                           \
  ({                                                                           \
    ty *ptr = (ty *)malloc(sizeof(ty));                                        \
    if (ptr != NULL) {                                                         \
      *ptr = (ty){__VA_ARGS__};                                                \
    }                                                                          \
    ptr;                                                                       \
  })

#define IMPL(class, ty, name, params, body)                                    \
  ty class##_##name(class *self, union params _) body;

#define USING(ty, body, ...)                                                   \
  do {                                                                         \
    ty *ty = (ty *)malloc(sizeof(ty));                                         \
    *ty = (ty)__VA_ARGS__;                                                     \
    body;                                                                      \
  } while (0)

#define CLASS(name, ...)                                                       \
  typedef struct name __VA_ARGS__ name;                                        \
                                                                               \
  typedef struct {                                                             \
    name *(*init)();                                                           \
    void (*destroy)(name * ptr);                                               \
  } name##Mgr;                                                                 \
                                                                               \
  name *name##_init() {                                                        \
    name *ptr = (name *)malloc(sizeof(name));                                  \
    return ptr;                                                                \
  }                                                                            \
                                                                               \
  void name##_destroy(name *ptr) { free(ptr); }                                \
                                                                               \
  name##Mgr *name##_mgr_init() {                                               \
    name##Mgr *mgr = (name##Mgr *)malloc(sizeof(name##Mgr));                   \
    mgr->init = name##_init;                                                   \
    mgr->destroy = name##_destroy;                                             \
    return mgr;                                                                \
  }                                                                            \
  struct name

#define SCOPED(ty, v, s)                                                       \
  ({                                                                           \
    scoped_t *ptr = (scoped_t *)malloc(s * sizeof(scoped_t));                  \
    if (ptr != NULL) {                                                         \
      *ptr = (scoped_t){ty, (type_t)v};                                        \
    }                                                                          \
    ptr;                                                                       \
  })

typedef enum { PRIVATE, PUBLIC } scoped_type_t;

typedef int *type_t;

typedef struct Scoped {
  scoped_type_t ty;
  type_t v;
} scoped_t;

type_t scoped_get(scoped_t *s);

void scoped_set(scoped_t *s, type_t v);

#endif
